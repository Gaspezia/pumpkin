use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::serverbound::play::CUSTOM_CLICK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Vanilla `ServerboundCustomClickActionPacket` payload bound (`lengthPrefixed(65536)`).
const MAX_PAYLOAD_LEN: usize = 65536;

/// Reads the optional NBT payload of a custom click action.
///
/// 26.2+: `VarInt` length followed by that many bytes (an unnamed NBT tree, or a single `00`
/// for "no payload"). Earlier versions: a bool followed by the raw NBT.
pub fn read_custom_click_payload<'a>(
    bytebuf: &mut &'a [u8],
    version: &JavaMinecraftVersion,
) -> Result<Option<&'a [u8]>, ReadingError> {
    if *version >= JavaMinecraftVersion::V_26_2 {
        let len = bytebuf.get_var_int()?.0;
        let len = usize::try_from(len)
            .ok()
            .filter(|len| *len <= MAX_PAYLOAD_LEN)
            .ok_or_else(|| ReadingError::TooLarge("Custom click payload too long".to_string()))?;
        let payload = bytebuf.read_slice_borrowed(len)?;
        Ok(if payload.is_empty() || payload == [0] {
            None
        } else {
            Some(payload)
        })
    } else {
        bytebuf.get_option(|b| b.read_remaining_slice_borrowed(32767))
    }
}

/// Inverse of [`read_custom_click_payload`].
pub fn write_custom_click_payload(
    write: &mut impl std::io::Write,
    payload: Option<&[u8]>,
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if *version >= JavaMinecraftVersion::V_26_2 {
        if let Some(payload) = payload {
            let len = i32::try_from(payload.len())
                .map_err(|_| WritingError::Message("Payload too long".to_string()))?;
            write.write_var_int(&VarInt(len))?;
            write.write_slice(payload)
        } else {
            write.write_var_int(&VarInt(1))?;
            write.write_u8(0)
        }
    } else if let Some(payload) = payload {
        write.write_bool(true)?;
        write.write_slice(payload)
    } else {
        write.write_bool(false)
    }
}

#[java_packet(CUSTOM_CLICK_ACTION)]
pub struct SCustomClickAction<'a> {
    pub action_id: &'a str,
    pub payload: Option<&'a [u8]>,
}

impl<'a> ServerPacket<'a> for SCustomClickAction<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            action_id: bytebuf.get_str_borrowed()?,
            payload: read_custom_click_payload(bytebuf, version)?,
        })
    }
}

impl crate::ClientPacket for SCustomClickAction<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(self.action_id)?;
        write_custom_click_payload(&mut write, self.payload, version)
    }
}

#[cfg(test)]
mod custom_click_tests {
    use super::*;

    fn bytes(id: &str, payload: Option<&[u8]>, v: JavaMinecraftVersion) -> Vec<u8> {
        let mut out = Vec::new();
        out.write_string(id).unwrap();
        write_custom_click_payload(&mut out, payload, &v).unwrap();
        out
    }

    fn read(raw: &[u8], v: JavaMinecraftVersion) -> Option<Vec<u8>> {
        let mut buf = raw;
        let p = SCustomClickAction::read(&mut buf, &v).unwrap();
        assert_eq!(p.action_id, "a:b");
        p.payload.map(<[u8]>::to_vec)
    }

    #[test]
    fn new_format_absent_short_long() {
        let v = JavaMinecraftVersion::V_26_3;
        // absent: id, VarInt 1, 00
        let mut raw = Vec::new();
        raw.write_string("a:b").unwrap();
        raw.extend([1, 0]);
        assert_eq!(read(&raw, v), None);
        assert_eq!(read(&bytes("a:b", None, v), v), None);

        let short = vec![10, 0, 0];
        assert_eq!(read(&bytes("a:b", Some(&short), v), v), Some(short));

        let mut long = vec![10];
        long.extend(std::iter::repeat_n(7u8, 300));
        assert_eq!(read(&bytes("a:b", Some(&long), v), v), Some(long));
        // 26.2 uses the same format
        let v2 = JavaMinecraftVersion::V_26_2;
        let long2 = vec![9u8; 200];
        assert_eq!(read(&bytes("a:b", Some(&long2), v2), v2), Some(long2));
    }

    #[test]
    fn new_format_rejects_oversize_and_truncated() {
        let v = JavaMinecraftVersion::V_26_3;
        let mut raw = Vec::new();
        raw.write_string("a:b").unwrap();
        raw.write_var_int(&VarInt(65537)).unwrap();
        assert!(SCustomClickAction::read(&mut &raw[..], &v).is_err());
        let mut raw = Vec::new();
        raw.write_string("a:b").unwrap();
        raw.extend([5, 1, 2]);
        assert!(SCustomClickAction::read(&mut &raw[..], &v).is_err());
    }

    #[test]
    fn old_format_bool_then_nbt() {
        let v = JavaMinecraftVersion::V_26_1;
        assert_eq!(read(&bytes("a:b", None, v), v), None);
        let short = vec![10, 0, 0];
        assert_eq!(read(&bytes("a:b", Some(&short), v), v), Some(short));
        let long = vec![5u8; 300];
        assert_eq!(read(&bytes("a:b", Some(&long), v), v), Some(long));
    }
}
