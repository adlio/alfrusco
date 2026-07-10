{-# LANGUAGE OverloadedStrings #-}

-- | Icon type and macOS system icon constants for Alfred workflows.
module Alfrusco.Icon
  ( Icon (..)
  , iconFromImage
  , iconForFiletype
    -- * Icon path root
  , iconRoot
    -- * System icon constants
  , iconArDocument
  , iconArObject
  , iconAccounts
  , iconActions
  , iconAirdrop
  , iconAlertCautionBadge
  , iconAlertNote
  , iconAlertStop
  , iconAliasBadge
  , iconAllMyFiles
  , iconAppleTraceFile
  , iconApplicationsFolder
  , iconBackwardArrow
  , iconBonjour
  , iconBookmark
  , iconBurnableFolder
  , iconBurning
  , iconCdAudioVolume
  , iconClippingPicture
  , iconClippingSound
  , iconClippingText
  , iconClippingUnknown
  , iconClock
  , iconColorSyncProfile
  , iconConnectTo
  , iconDesktopFolder
  , iconDeveloperFolder
  , iconDocumentsFolder
  , iconDownloadsFolder
  , iconDropFolderBadge
  , iconEjectMedia
  , iconErasing
  , iconEveryone
  , iconExecutableBinary
  , iconFavoriteItems
  , iconFileVault
  , iconFinder
  , iconForwardArrow
  , iconFullTrash
  , iconGeneral
  , iconGenericAirDisk
  , iconGenericApplication
  , iconGenericDocument
  , iconGenericFileServer
  , iconGenericFolder
  , iconGenericFont
  , iconGenericNetwork
  , iconGenericQuestionMark
  , iconGenericSharepoint
  , iconGenericSpeaker
  , iconGenericStationery
  , iconGenericTimeMachineDisk
  , iconGenericUrl
  , iconGenericWindow
  , iconGrid
  , iconGroupFolder
  , iconGroup
  , iconGuestUser
  , iconHelp
  , iconHomeFolder
  , iconInternetLocation
  , iconKext
  , iconKeepArranged
  , iconLibraryFolder
  , iconLockedBadge
  , iconLocked
  , iconMagnifyingGlass
  , iconMovieFolder
  , iconMultipleItems
  , iconMusicFolder
  , iconNetBootVolume
  , iconNewFolderBadge
  , iconNoWrite
  , iconNotLoaded
  , iconNotifications
  , iconOpenFolder
  , iconPicturesFolder
  , iconPrivateFolderBadge
  , iconProblemReport
  , iconProfileBackgroundColor
  , iconProfileFont
  , iconProfileFontAndColor
  , iconPublicFolder
  , iconReadOnlyFolderBadge
  , iconRealityFile
  , iconRecentItems
  , iconRightContainerArrow
  , iconServerApplicationsFolder
  , iconSitesFolder
  , iconSmartFolder
  , iconSync
  , iconSystemFolder
  , iconToolbarAdvanced
  , iconToolbarCustomize
  , iconToolbarDelete
  , iconToolbarFavorites
  , iconToolbarInfo
  , iconToolbarLabels
  , iconTrash
  , iconUnknownFsObject
  , iconUnlocked
  , iconUnsupported
  , iconUser
  , iconUserUnknown
  , iconUsersFolder
  , iconUtilitiesFolder
  ) where

import Data.Aeson (ToJSON (..), (.=))
import Data.Aeson qualified as Aeson
import Data.Text (Text)

-- | Represents an icon in an Alfred item.
data Icon = Icon
  { iconType :: Maybe Text
  , iconPath :: Text
  }
  deriving (Show, Eq)

instance ToJSON Icon where
  toJSON (Icon Nothing path) =
    Aeson.object ["path" .= path]
  toJSON (Icon (Just ty) path) =
    Aeson.object ["type" .= ty, "path" .= path]

-- | Create an icon from an image path (no type field).
iconFromImage :: Text -> Icon
iconFromImage path = Icon {iconType = Nothing, iconPath = path}

-- | Create an icon for a filetype (sets type to "filetype").
iconForFiletype :: Text -> Icon
iconForFiletype path = Icon {iconType = Just "filetype", iconPath = path}

-- | Root path for macOS system icons.
iconRoot :: Text
iconRoot = "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources"

mkIcon :: Text -> Icon
mkIcon path = Icon {iconType = Nothing, iconPath = path}

iconArDocument :: Icon
iconArDocument = mkIcon (iconRoot <> "/ARDocument.icns")

iconArObject :: Icon
iconArObject = mkIcon (iconRoot <> "/ARObject.icns")

iconAccounts :: Icon
iconAccounts = mkIcon (iconRoot <> "/Accounts.icns")

iconActions :: Icon
iconActions = mkIcon (iconRoot <> "/Actions.icns")

iconAirdrop :: Icon
iconAirdrop = mkIcon (iconRoot <> "/AirDrop.icns")

iconAlertCautionBadge :: Icon
iconAlertCautionBadge = mkIcon (iconRoot <> "/AlertCautionBadgeIcon.icns")

iconAlertNote :: Icon
iconAlertNote = mkIcon (iconRoot <> "/AlertNoteIcon.icns")

iconAlertStop :: Icon
iconAlertStop = mkIcon (iconRoot <> "/AlertStopIcon.icns")

iconAliasBadge :: Icon
iconAliasBadge = mkIcon (iconRoot <> "/AliasBadgeIcon.icns")

iconAllMyFiles :: Icon
iconAllMyFiles = mkIcon (iconRoot <> "/AllMyFiles.icns")

iconAppleTraceFile :: Icon
iconAppleTraceFile = mkIcon (iconRoot <> "/AppleTraceFile.icns")

iconApplicationsFolder :: Icon
iconApplicationsFolder = mkIcon (iconRoot <> "/ApplicationsFolderIcon.icns")

iconBackwardArrow :: Icon
iconBackwardArrow = mkIcon (iconRoot <> "/BackwardArrowIcon.icns")

iconBonjour :: Icon
iconBonjour = mkIcon (iconRoot <> "/Bonjour.icns")

iconBookmark :: Icon
iconBookmark = mkIcon (iconRoot <> "/BookmarkIcon.icns")

iconBurnableFolder :: Icon
iconBurnableFolder = mkIcon (iconRoot <> "/BurnableFolderIcon.icns")

iconBurning :: Icon
iconBurning = mkIcon (iconRoot <> "/BurningIcon.icns")

iconCdAudioVolume :: Icon
iconCdAudioVolume = mkIcon (iconRoot <> "/CDAudioVolumeIcon.icns")

iconClippingPicture :: Icon
iconClippingPicture = mkIcon (iconRoot <> "/ClippingPicture.icns")

iconClippingSound :: Icon
iconClippingSound = mkIcon (iconRoot <> "/ClippingSound.icns")

iconClippingText :: Icon
iconClippingText = mkIcon (iconRoot <> "/ClippingText.icns")

iconClippingUnknown :: Icon
iconClippingUnknown = mkIcon (iconRoot <> "/ClippingUnknown.icns")

iconClock :: Icon
iconClock = mkIcon (iconRoot <> "/Clock.icns")

iconColorSyncProfile :: Icon
iconColorSyncProfile = mkIcon (iconRoot <> "/ColorSyncProfileIcon.icns")

iconConnectTo :: Icon
iconConnectTo = mkIcon (iconRoot <> "/ConnectToIcon.icns")

iconDesktopFolder :: Icon
iconDesktopFolder = mkIcon (iconRoot <> "/DesktopFolderIcon.icns")

iconDeveloperFolder :: Icon
iconDeveloperFolder = mkIcon (iconRoot <> "/DeveloperFolderIcon.icns")

iconDocumentsFolder :: Icon
iconDocumentsFolder = mkIcon (iconRoot <> "/DocumentsFolderIcon.icns")

iconDownloadsFolder :: Icon
iconDownloadsFolder = mkIcon (iconRoot <> "/DownloadsFolder.icns")

iconDropFolderBadge :: Icon
iconDropFolderBadge = mkIcon (iconRoot <> "/DropFolderBadgeIcon.icns")

iconEjectMedia :: Icon
iconEjectMedia = mkIcon (iconRoot <> "/EjectMediaIcon.icns")

iconErasing :: Icon
iconErasing = mkIcon (iconRoot <> "/ErasingIcon.icns")

iconEveryone :: Icon
iconEveryone = mkIcon (iconRoot <> "/Everyone.icns")

iconExecutableBinary :: Icon
iconExecutableBinary = mkIcon (iconRoot <> "/ExecutableBinaryIcon.icns")

iconFavoriteItems :: Icon
iconFavoriteItems = mkIcon (iconRoot <> "/FavoriteItemsIcon.icns")

iconFileVault :: Icon
iconFileVault = mkIcon (iconRoot <> "/FileVaultIcon.icns")

iconFinder :: Icon
iconFinder = mkIcon (iconRoot <> "/FinderIcon.icns")

iconForwardArrow :: Icon
iconForwardArrow = mkIcon (iconRoot <> "/ForwardArrowIcon.icns")

iconFullTrash :: Icon
iconFullTrash = mkIcon (iconRoot <> "/FullTrashIcon.icns")

iconGeneral :: Icon
iconGeneral = mkIcon (iconRoot <> "/General.icns")

iconGenericAirDisk :: Icon
iconGenericAirDisk = mkIcon (iconRoot <> "/GenericAirDiskIcon.icns")

iconGenericApplication :: Icon
iconGenericApplication = mkIcon (iconRoot <> "/GenericApplicationIcon.icns")

iconGenericDocument :: Icon
iconGenericDocument = mkIcon (iconRoot <> "/GenericDocumentIcon.icns")

iconGenericFileServer :: Icon
iconGenericFileServer = mkIcon (iconRoot <> "/GenericFileServerIcon.icns")

iconGenericFolder :: Icon
iconGenericFolder = mkIcon (iconRoot <> "/GenericFolderIcon.icns")

iconGenericFont :: Icon
iconGenericFont = mkIcon (iconRoot <> "/GenericFontIcon.icns")

iconGenericNetwork :: Icon
iconGenericNetwork = mkIcon (iconRoot <> "/GenericNetworkIcon.icns")

iconGenericQuestionMark :: Icon
iconGenericQuestionMark = mkIcon (iconRoot <> "/GenericQuestionMarkIcon.icns")

iconGenericSharepoint :: Icon
iconGenericSharepoint = mkIcon (iconRoot <> "/GenericSharepoint.icns")

iconGenericSpeaker :: Icon
iconGenericSpeaker = mkIcon (iconRoot <> "/GenericSpeaker.icns")

iconGenericStationery :: Icon
iconGenericStationery = mkIcon (iconRoot <> "/GenericStationeryIcon.icns")

iconGenericTimeMachineDisk :: Icon
iconGenericTimeMachineDisk = mkIcon (iconRoot <> "/GenericTimeMachineDiskIcon.icns")

iconGenericUrl :: Icon
iconGenericUrl = mkIcon (iconRoot <> "/GenericURLIcon.icns")

iconGenericWindow :: Icon
iconGenericWindow = mkIcon (iconRoot <> "/GenericWindowIcon.icns")

iconGrid :: Icon
iconGrid = mkIcon (iconRoot <> "/GridIcon.icns")

iconGroupFolder :: Icon
iconGroupFolder = mkIcon (iconRoot <> "/GroupFolder.icns")

iconGroup :: Icon
iconGroup = mkIcon (iconRoot <> "/GroupIcon.icns")

iconGuestUser :: Icon
iconGuestUser = mkIcon (iconRoot <> "/GuestUserIcon.icns")

iconHelp :: Icon
iconHelp = mkIcon (iconRoot <> "/HelpIcon.icns")

iconHomeFolder :: Icon
iconHomeFolder = mkIcon (iconRoot <> "/HomeFolderIcon.icns")

iconInternetLocation :: Icon
iconInternetLocation = mkIcon (iconRoot <> "/InternetLocation.icns")

iconKext :: Icon
iconKext = mkIcon (iconRoot <> "/KEXT.icns")

iconKeepArranged :: Icon
iconKeepArranged = mkIcon (iconRoot <> "/KeepArrangedIcon.icns")

iconLibraryFolder :: Icon
iconLibraryFolder = mkIcon (iconRoot <> "/LibraryFolderIcon.icns")

iconLockedBadge :: Icon
iconLockedBadge = mkIcon (iconRoot <> "/LockedBadgeIcon.icns")

iconLocked :: Icon
iconLocked = mkIcon (iconRoot <> "/LockedIcon.icns")

iconMagnifyingGlass :: Icon
iconMagnifyingGlass = mkIcon (iconRoot <> "/MagnifyingGlassIcon.icns")

iconMovieFolder :: Icon
iconMovieFolder = mkIcon (iconRoot <> "/MovieFolderIcon.icns")

iconMultipleItems :: Icon
iconMultipleItems = mkIcon (iconRoot <> "/MultipleItemsIcon.icns")

iconMusicFolder :: Icon
iconMusicFolder = mkIcon (iconRoot <> "/MusicFolderIcon.icns")

iconNetBootVolume :: Icon
iconNetBootVolume = mkIcon (iconRoot <> "/NetBootVolume.icns")

iconNewFolderBadge :: Icon
iconNewFolderBadge = mkIcon (iconRoot <> "/NewFolderBadgeIcon.icns")

iconNoWrite :: Icon
iconNoWrite = mkIcon (iconRoot <> "/NoWriteIcon.icns")

iconNotLoaded :: Icon
iconNotLoaded = mkIcon (iconRoot <> "/NotLoaded.icns")

iconNotifications :: Icon
iconNotifications = mkIcon (iconRoot <> "/Notifications.icns")

iconOpenFolder :: Icon
iconOpenFolder = mkIcon (iconRoot <> "/OpenFolderIcon.icns")

iconPicturesFolder :: Icon
iconPicturesFolder = mkIcon (iconRoot <> "/PicturesFolderIcon.icns")

iconPrivateFolderBadge :: Icon
iconPrivateFolderBadge = mkIcon (iconRoot <> "/PrivateFolderBadgeIcon.icns")

iconProblemReport :: Icon
iconProblemReport = mkIcon (iconRoot <> "/ProblemReport.icns")

iconProfileBackgroundColor :: Icon
iconProfileBackgroundColor = mkIcon (iconRoot <> "/ProfileBackgroundColor.icns")

iconProfileFont :: Icon
iconProfileFont = mkIcon (iconRoot <> "/ProfileFont.icns")

iconProfileFontAndColor :: Icon
iconProfileFontAndColor = mkIcon (iconRoot <> "/ProfileFontAndColor.icns")

iconPublicFolder :: Icon
iconPublicFolder = mkIcon (iconRoot <> "/PublicFolderIcon.icns")

iconReadOnlyFolderBadge :: Icon
iconReadOnlyFolderBadge = mkIcon (iconRoot <> "/ReadOnlyFolderBadgeIcon.icns")

iconRealityFile :: Icon
iconRealityFile = mkIcon (iconRoot <> "/RealityFile.icns")

iconRecentItems :: Icon
iconRecentItems = mkIcon (iconRoot <> "/RecentItemsIcon.icns")

iconRightContainerArrow :: Icon
iconRightContainerArrow = mkIcon (iconRoot <> "/RightContainerArrowIcon.icns")

iconServerApplicationsFolder :: Icon
iconServerApplicationsFolder = mkIcon (iconRoot <> "/ServerApplicationsFolderIcon.icns")

iconSitesFolder :: Icon
iconSitesFolder = mkIcon (iconRoot <> "/SitesFolderIcon.icns")

iconSmartFolder :: Icon
iconSmartFolder = mkIcon (iconRoot <> "/SmartFolderIcon.icns")

iconSync :: Icon
iconSync = mkIcon (iconRoot <> "/Sync.icns")

iconSystemFolder :: Icon
iconSystemFolder = mkIcon (iconRoot <> "/SystemFolderIcon.icns")

iconToolbarAdvanced :: Icon
iconToolbarAdvanced = mkIcon (iconRoot <> "/ToolbarAdvanced.icns")

iconToolbarCustomize :: Icon
iconToolbarCustomize = mkIcon (iconRoot <> "/ToolbarCustomizeIcon.icns")

iconToolbarDelete :: Icon
iconToolbarDelete = mkIcon (iconRoot <> "/ToolbarDeleteIcon.icns")

iconToolbarFavorites :: Icon
iconToolbarFavorites = mkIcon (iconRoot <> "/ToolbarFavoritesIcon.icns")

iconToolbarInfo :: Icon
iconToolbarInfo = mkIcon (iconRoot <> "/ToolbarInfo.icns")

iconToolbarLabels :: Icon
iconToolbarLabels = mkIcon (iconRoot <> "/ToolbarLabels.icns")

iconTrash :: Icon
iconTrash = mkIcon (iconRoot <> "/TrashIcon.icns")

iconUnknownFsObject :: Icon
iconUnknownFsObject = mkIcon (iconRoot <> "/UnknownFSObjectIcon.icns")

iconUnlocked :: Icon
iconUnlocked = mkIcon (iconRoot <> "/UnlockedIcon.icns")

iconUnsupported :: Icon
iconUnsupported = mkIcon (iconRoot <> "/Unsupported.icns")

iconUser :: Icon
iconUser = mkIcon (iconRoot <> "/UserIcon.icns")

iconUserUnknown :: Icon
iconUserUnknown = mkIcon (iconRoot <> "/UserUnknownIcon.icns")

iconUsersFolder :: Icon
iconUsersFolder = mkIcon (iconRoot <> "/UsersFolderIcon.icns")

iconUtilitiesFolder :: Icon
iconUtilitiesFolder = mkIcon (iconRoot <> "/UtilitiesFolder.icns")
