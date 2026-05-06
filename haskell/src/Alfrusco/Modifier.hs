{-# LANGUAGE OverloadedStrings #-}

-- | Modifier keys and modifier records for Alfred items.
module Alfrusco.Modifier
  ( Key (..)
  , keyToText
  , keysToText
  , Modifier (..)
  , modifier
  , modifierCombo
  , withModSubtitle
  , withModArg
  , withModArgs
  , withModIcon
  , withModIconFromImage
  , withModIconForFiletype
  , withModVar
  , withModAutocomplete
  , withModValid
  ) where

import Data.Aeson (ToJSON (..), (.=))
import Data.Aeson qualified as Aeson
import Data.Map.Strict (Map)
import Data.Map.Strict qualified as Map
import Data.Text (Text)
import Data.Text qualified as Text

import Alfrusco.Arg (Arg (..))
import Alfrusco.Icon (Icon (..), iconForFiletype, iconFromImage)

-- | Represents one of the modifier keys.
data Key = Cmd | Ctrl | Alt | Shift | Fn
  deriving (Show, Eq)

-- | Convert a Key to its text representation.
keyToText :: Key -> Text
keyToText Cmd = "cmd"
keyToText Ctrl = "ctrl"
keyToText Alt = "alt"
keyToText Shift = "shift"
keyToText Fn = "fn"

-- | Convert a list of Keys to a combined text representation joined by "+".
keysToText :: [Key] -> Text
keysToText = Text.intercalate "+" . map keyToText

-- | Represents a modifier entry in an Alfred item's "mods" object.
data Modifier = Modifier
  { modKeys :: Text
  , modSubtitle :: Maybe Text
  , modArg :: Maybe Arg
  , modIcon :: Maybe Icon
  , modVariables :: Maybe (Map Text Text)
  , modAutocomplete :: Maybe Text
  , modValid :: Maybe Bool
  }
  deriving (Show, Eq)

instance ToJSON Modifier where
  toJSON m =
    Aeson.object $ concat
      [ maybe [] (\s -> ["subtitle" .= s]) (modSubtitle m)
      , maybe [] (\a -> ["arg" .= a]) (modArg m)
      , maybe [] (\i -> ["icon" .= i]) (modIcon m)
      , maybe [] (\v -> ["variables" .= v]) (modVariables m)
      , maybe [] (\ac -> ["autocomplete" .= ac]) (modAutocomplete m)
      , maybe [] (\v -> ["valid" .= v]) (modValid m)
      ]

-- | Create a Modifier for a single key.
modifier :: Key -> Modifier
modifier k = Modifier
  { modKeys = keyToText k
  , modSubtitle = Nothing
  , modArg = Nothing
  , modIcon = Nothing
  , modVariables = Nothing
  , modAutocomplete = Nothing
  , modValid = Nothing
  }

-- | Create a Modifier for a combination of keys.
modifierCombo :: [Key] -> Modifier
modifierCombo ks = Modifier
  { modKeys = keysToText ks
  , modSubtitle = Nothing
  , modArg = Nothing
  , modIcon = Nothing
  , modVariables = Nothing
  , modAutocomplete = Nothing
  , modValid = Nothing
  }

-- | Set the subtitle on a Modifier.
withModSubtitle :: Text -> Modifier -> Modifier
withModSubtitle s m = m {modSubtitle = Just s}

-- | Set a single argument on a Modifier.
withModArg :: Text -> Modifier -> Modifier
withModArg a m = m {modArg = Just (ArgOne a)}

-- | Set multiple arguments on a Modifier.
withModArgs :: [Text] -> Modifier -> Modifier
withModArgs as' m = m {modArg = Just (ArgMany as')}

-- | Set the icon on a Modifier.
withModIcon :: Icon -> Modifier -> Modifier
withModIcon i m = m {modIcon = Just i}

-- | Set an icon from an image path on a Modifier.
withModIconFromImage :: Text -> Modifier -> Modifier
withModIconFromImage path m = m {modIcon = Just (iconFromImage path)}

-- | Set an icon for a filetype on a Modifier.
withModIconForFiletype :: Text -> Modifier -> Modifier
withModIconForFiletype ft m = m {modIcon = Just (iconForFiletype ft)}

-- | Add a variable to a Modifier.
withModVar :: Text -> Text -> Modifier -> Modifier
withModVar k v m = m {modVariables = Just updated}
  where
    updated = Map.insert k v (maybe Map.empty id (modVariables m))

-- | Set the autocomplete on a Modifier.
withModAutocomplete :: Text -> Modifier -> Modifier
withModAutocomplete ac m = m {modAutocomplete = Just ac}

-- | Set the valid flag on a Modifier.
withModValid :: Bool -> Modifier -> Modifier
withModValid v m = m {modValid = Just v}
