use std::io::{Read, Write};

use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::input::{KeyInput, MouseInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;

use crate::errors::CaptureError;
use crate::services::field_reader::FieldReader;
use crate::structs::{CaptureEvent, CaptureHeader, FrameInput, SubmitStatus, TickInput};

// .lmcap: a 20-byte header and the terrain config as JSON, then records of
// [tag u8][payload length u32][payload], zstd-compressed (or raw, for the C++ e2e harness)
pub struct CaptureCodec;

impl CaptureCodec {
    pub const MAGIC: &'static [u8; 4] = b"LMCP";
    pub const VERSION: u16 = 2;
    pub const RAW: u8 = 0;
    pub const ZSTD: u8 = 1;
    pub const HEADER_BYTES: usize = 20;
    pub const RECORD_HEADER_BYTES: usize = 5;
    pub const ENTER: u8 = 1;
    pub const LEAVE: u8 = 2;
    pub const TICK: u8 = 3;
    pub const SUBMIT: u8 = 4;
    pub const COLUMN: u8 = 5;
    pub const BLOCK: u8 = 6;
    pub const KEY: u8 = 7;
    pub const MOUSE: u8 = 8;
    pub const FRAME: u8 = 9;
    pub const BIOME_NAME: u8 = 10;
    // a biome name record holds this many name bytes; a longer name is cut
    pub const NAME_BYTES: usize = 64;
    const PROBE_BYTES: usize = std::mem::size_of::<ChunkProbe>();

    pub fn write_header(out: &mut impl Write, header: &CaptureHeader, compression: u8) -> Result<(), CaptureError> {
        let json = serde_json::to_vec(&header.terrain)?;
        out.write_all(Self::MAGIC)?;
        out.write_all(&Self::VERSION.to_le_bytes())?;
        out.write_all(&[compression, 0])?;
        out.write_all(&header.settings_hash.to_le_bytes())?;
        out.write_all(&(json.len() as u32).to_le_bytes())?;
        out.write_all(&json)?;
        Ok(())
    }

    // returns the header and the compression of the records that follow
    pub fn read_header(input: &mut impl Read) -> Result<(CaptureHeader, u8), CaptureError> {
        let mut fixed = [0u8; Self::HEADER_BYTES];
        input.read_exact(&mut fixed)?;
        if &fixed[0..4] != Self::MAGIC {
            return Err(CaptureError::BadMagic);
        }
        let mut fields = FieldReader::new(&fixed[4..]);
        let version = u16::from_le_bytes([fields.u8(), fields.u8()]);
        if version != Self::VERSION {
            return Err(CaptureError::UnsupportedVersion(version));
        }
        let compression = fields.u8();
        if compression != Self::RAW && compression != Self::ZSTD {
            return Err(CaptureError::UnsupportedCompression(compression));
        }
        let _reserved = fields.u8();
        let low = u64::from(fields.u32());
        let settings_hash = low | (u64::from(fields.u32()) << 32);
        let mut json = vec![0u8; fields.u32() as usize];
        input.read_exact(&mut json)?;
        Ok((CaptureHeader { settings_hash, terrain: serde_json::from_slice(&json)? }, compression))
    }

    // appends one whole record to `out`
    pub fn encode(event: &CaptureEvent, out: &mut Vec<u8>) {
        let start = out.len();
        out.extend_from_slice(&[Self::tag(event), 0, 0, 0, 0]);
        match event {
            CaptureEvent::Enter { dim } => out.extend_from_slice(&dim.to_le_bytes()),
            CaptureEvent::Leave => {}
            CaptureEvent::Tick(input) => Self::put_tick(out, input),
            CaptureEvent::Submit { pos, status, probe } => {
                Self::put_pos(out, *pos);
                out.push(*status as u8);
                out.extend_from_slice(bytemuck::bytes_of(probe.as_ref()));
            }
            CaptureEvent::BiomeName { id, name } => Self::put_name(out, *id, name),
            CaptureEvent::Column { pos, index, sample } => {
                Self::put_pos(out, *pos);
                out.extend_from_slice(&index.to_le_bytes());
                out.extend_from_slice(&sample.surface.to_le_bytes());
                out.extend_from_slice(&sample.water.to_le_bytes());
                out.extend_from_slice(&sample.height.to_le_bytes());
                out.extend_from_slice(&sample.solid.to_le_bytes());
                out.push(u8::from(sample.retry));
                out.extend_from_slice(&sample.biome_id.to_le_bytes());
            }
            CaptureEvent::Block { x, z, dim } => Self::put_pos(out, ChunkPos::new(*x, *z, *dim)),
            CaptureEvent::Key(key) => {
                out.extend_from_slice(&key.code.to_le_bytes());
                out.extend_from_slice(&[u8::from(key.down), u8::from(key.ctrl), u8::from(key.mouse_grabbed)]);
            }
            CaptureEvent::Mouse(mouse) => {
                out.extend_from_slice(&mouse.button.to_le_bytes());
                out.extend_from_slice(&mouse.data.to_le_bytes());
            }
            CaptureEvent::Frame(frame) => Self::put_frame(out, frame),
        }
        let len = (out.len() - start - Self::RECORD_HEADER_BYTES) as u32;
        out[start + 1..start + 5].copy_from_slice(&len.to_le_bytes());
    }

    pub fn decode(tag: u8, payload: &[u8]) -> Result<CaptureEvent, CaptureError> {
        let expected = Self::payload_bytes(tag).ok_or(CaptureError::UnknownTag(tag))?;
        if payload.len() != expected {
            return Err(CaptureError::WrongSize { tag, expected, actual: payload.len() });
        }
        let mut f = FieldReader::new(payload);
        let event = match tag {
            Self::ENTER => CaptureEvent::Enter { dim: f.i32() },
            Self::LEAVE => CaptureEvent::Leave,
            Self::TICK => CaptureEvent::Tick(Self::read_tick(&mut f)),
            Self::SUBMIT => {
                let pos = Self::read_pos(&mut f);
                let status = SubmitStatus::from_code(f.u8()).ok_or(CaptureError::BadValue(tag))?;
                let probe: ChunkProbe = bytemuck::pod_read_unaligned(f.rest());
                CaptureEvent::Submit { pos, status, probe: Box::new(probe) }
            }
            Self::COLUMN => {
                let pos = Self::read_pos(&mut f);
                let index = f.u32();
                if index as usize >= ChunkSurface::COLUMNS {
                    return Err(CaptureError::BadValue(tag));
                }
                let sample = ColumnSample::new(f.u32(), f.u32(), f.i16(), f.i16(), f.flag(), f.u16());
                CaptureEvent::Column { pos, index, sample }
            }
            Self::BLOCK => CaptureEvent::Block { x: f.i32(), z: f.i32(), dim: f.i32() },
            Self::KEY => CaptureEvent::Key(KeyInput::new(f.i32(), f.flag(), f.flag(), f.flag())),
            Self::MOUSE => CaptureEvent::Mouse(MouseInput::new(f.i32(), f.i32())),
            Self::BIOME_NAME => Self::read_name(&mut f)?,
            _ => CaptureEvent::Frame(Self::read_frame(&mut f)),
        };
        Ok(event)
    }

    pub fn payload_bytes(tag: u8) -> Option<usize> {
        match tag {
            Self::ENTER => Some(4),
            Self::LEAVE => Some(0),
            Self::TICK => Some(23),
            Self::SUBMIT => Some(13 + Self::PROBE_BYTES),
            Self::COLUMN => Some(31),
            Self::BLOCK => Some(12),
            Self::KEY => Some(7),
            Self::MOUSE => Some(8),
            Self::FRAME => Some(22),
            Self::BIOME_NAME => Some(3 + Self::NAME_BYTES),
            _ => None,
        }
    }

    fn tag(event: &CaptureEvent) -> u8 {
        match event {
            CaptureEvent::Enter { .. } => Self::ENTER,
            CaptureEvent::Leave => Self::LEAVE,
            CaptureEvent::Tick(_) => Self::TICK,
            CaptureEvent::Submit { .. } => Self::SUBMIT,
            CaptureEvent::Column { .. } => Self::COLUMN,
            CaptureEvent::Block { .. } => Self::BLOCK,
            CaptureEvent::Key(_) => Self::KEY,
            CaptureEvent::Mouse(_) => Self::MOUSE,
            CaptureEvent::Frame(_) => Self::FRAME,
            CaptureEvent::BiomeName { .. } => Self::BIOME_NAME,
        }
    }

    // id, length, then the name in a fixed field; a longer name is cut at a character boundary
    fn put_name(out: &mut Vec<u8>, id: u16, name: &str) {
        let mut len = name.len().min(Self::NAME_BYTES);
        while !name.is_char_boundary(len) {
            len -= 1;
        }
        out.extend_from_slice(&id.to_le_bytes());
        out.push(len as u8);
        out.extend_from_slice(&name.as_bytes()[..len]);
        out.resize(out.len() + Self::NAME_BYTES - len, 0);
    }

    fn read_name(f: &mut FieldReader<'_>) -> Result<CaptureEvent, CaptureError> {
        let id = f.u16();
        let len = usize::from(f.u8());
        let bytes = f.rest();
        if len > Self::NAME_BYTES {
            return Err(CaptureError::BadValue(Self::BIOME_NAME));
        }
        let name = std::str::from_utf8(&bytes[..len]).map_err(|_| CaptureError::BadValue(Self::BIOME_NAME))?;
        Ok(CaptureEvent::BiomeName { id, name: name.to_string() })
    }

    fn put_pos(out: &mut Vec<u8>, pos: ChunkPos) {
        out.extend_from_slice(&pos.x.to_le_bytes());
        out.extend_from_slice(&pos.z.to_le_bytes());
        out.extend_from_slice(&pos.dim.to_le_bytes());
    }

    fn read_pos(f: &mut FieldReader<'_>) -> ChunkPos {
        ChunkPos::new(f.i32(), f.i32(), f.i32())
    }

    fn put_tick(out: &mut Vec<u8>, input: &TickInput) {
        let p = &input.player;
        out.push(u8::from(p.present));
        for v in [p.x, p.y, p.z, p.yaw] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&p.dim.to_le_bytes());
        out.extend_from_slice(&[u8::from(input.foreground), u8::from(input.cursor_hidden)]);
    }

    fn read_tick(f: &mut FieldReader<'_>) -> TickInput {
        let player =
            PlayerSnapshot { present: f.flag(), x: f.f32(), y: f.f32(), z: f.f32(), yaw: f.f32(), dim: f.i32() };
        // captures do not record deaths
        TickInput { player, foreground: f.flag(), cursor_hidden: f.flag(), dead: false }
    }

    fn put_frame(out: &mut Vec<u8>, frame: &FrameInput) {
        for v in [frame.display.0, frame.display.1, frame.dt, frame.cursor.0, frame.cursor.1] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&[u8::from(frame.left_down), u8::from(frame.foreground)]);
    }

    fn read_frame(f: &mut FieldReader<'_>) -> FrameInput {
        FrameInput {
            display: (f.f32(), f.f32()),
            dt: f.f32(),
            cursor: (f.f32(), f.f32()),
            left_down: f.flag(),
            foreground: f.flag(),
            ..FrameInput::default()
        }
    }
}
