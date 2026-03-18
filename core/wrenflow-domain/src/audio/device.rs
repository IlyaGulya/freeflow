/// Cross-platform audio input device info.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}
