use serde::{Deserialize, Serialize};

pub const ICON_ROOT: &str = "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources";

pub const ICON_ACCOUNT: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/Accounts.icns";
pub const ICON_BURN: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/BurningIcon.icns";
pub const ICON_CLOCK: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/Clock.icns";
pub const ICON_COLOR: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/ProfileBackgroundColor.icns";
pub const ICON_EJECT: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/EjectMediaIcon.icns";
// Shown when a workflow throws an error
pub const ICON_ERROR: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/AlertStopIcon.icns";
pub const ICON_FAVORITE: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/ToolbarFavoritesIcon.icns";
pub const ICON_GROUP: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GroupIcon.icns";
pub const ICON_HELP: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/HelpIcon.icns";
pub const ICON_HOME: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/HomeFolderIcon.icns";
pub const ICON_INFO: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/ToolbarInfo.icns";
pub const ICON_NETWORK: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericNetworkIcon.icns";
pub const ICON_NOTE: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/AlertNoteIcon.icns";
pub const ICON_SETTINGS: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/ToolbarAdvanced.icns";
pub const ICON_SWIRL: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/ErasingIcon.icns";
pub const ICON_SWITCH: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/General.icns";
pub const ICON_SYNC: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/Sync.icns";
pub const ICON_TRASH: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/TrashIcon.icns";
pub const ICON_USER: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/UserIcon.icns";
pub const ICON_WARNING: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/AlertCautionIcon.icns";
pub const ICON_WEB: &str =
    "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/BookmarkIcon.icns";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub(crate) type_: Option<String>,

    pub(crate) path: String,
}
