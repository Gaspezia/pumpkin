use crate::{
    ServerPacket,
    java::server::play::{read_custom_click_payload, write_custom_click_payload},
    ser::{NetworkReadSliceExt, ReadingError, WritingError},
};
use pumpkin_data::packet::serverbound::config::CUSTOM_CLICK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

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
        use crate::ser::NetworkWriteExt;
        write.write_string(self.action_id)?;
        write_custom_click_payload(&mut write, self.payload, version)
    }
}
