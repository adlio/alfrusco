{-# LANGUAGE OverloadedStrings #-}

-- | Item type representing a single choice in the Alfred selection UI.
module Alfrusco.Item
  ( Item (..)
  , item
  , withSubtitle
  , withArg
  , withArgs
  , withVar
  , withUnsetVar
  , withUid
  , withValid
  , withIcon
  , withIconFromImage
  , withIconForFiletype
  , withModifier
  , withAutocomplete
  , withMatches
  , withQuicklookUrl
  , withCopyText
  , withLargeTypeText
  , withSticky
  , withBoost
    -- * Boost constants
  , boostSlight
  , boostLow
  , boostModerate
  , boostHigh
  , boostHigher
  , boostHighest
  ) where

import Data.Aeson (ToJSON (..), (.=))
import Data.Aeson qualified as Aeson
import Data.Map.Strict (Map)
import Data.Map.Strict qualified as Map
import Data.Text (Text)

import Alfrusco.Arg (Arg (..))
import Alfrusco.Icon (Icon (..), iconForFiletype, iconFromImage)
import Alfrusco.Modifier (Modifier (..))
import Alfrusco.Text (TextContent (..))

-- | Represents a single item in Alfred's output.
data Item = Item
  { itemTitle :: Text
  , itemSubtitle :: Maybe Text
  , itemUid :: Maybe Text
  , itemArg :: Maybe Arg
  , itemVariables :: Map Text Text
  , itemIcon :: Maybe Icon
  , itemValid :: Maybe Bool
  , itemMatch :: Maybe Text
  , itemModifiers :: Map Text Modifier
  , itemAutocomplete :: Maybe Text
  , itemQuicklookUrl :: Maybe Text
  , itemText :: Maybe TextContent
  , itemSticky :: Bool
  , itemBoost :: Int
  }
  deriving (Show, Eq)

instance ToJSON Item where
  toJSON i =
    Aeson.object $ concat
      [ ["title" .= itemTitle i]
      , maybe [] (\s -> ["subtitle" .= s]) (itemSubtitle i)
      , maybe [] (\u -> ["uid" .= u]) (itemUid i)
      , maybe [] (\a -> ["arg" .= a]) (itemArg i)
      , if Map.null (itemVariables i) then [] else ["variables" .= itemVariables i]
      , maybe [] (\ic -> ["icon" .= ic]) (itemIcon i)
      , maybe [] (\v -> ["valid" .= v]) (itemValid i)
      , maybe [] (\m -> ["match" .= m]) (itemMatch i)
      , if Map.null (itemModifiers i) then [] else ["mods" .= itemModifiers i]
      , maybe [] (\ac -> ["autocomplete" .= ac]) (itemAutocomplete i)
      , maybe [] (\q -> ["quicklookurl" .= q]) (itemQuicklookUrl i)
      , maybe [] (\t -> ["text" .= t]) (itemText i)
      ]

-- | Create a new Item with just a title. All other fields are empty/default.
item :: Text -> Item
item title = Item
  { itemTitle = title
  , itemSubtitle = Nothing
  , itemUid = Nothing
  , itemArg = Nothing
  , itemVariables = Map.empty
  , itemIcon = Nothing
  , itemValid = Nothing
  , itemMatch = Nothing
  , itemModifiers = Map.empty
  , itemAutocomplete = Nothing
  , itemQuicklookUrl = Nothing
  , itemText = Nothing
  , itemSticky = False
  , itemBoost = 0
  }

-- | Set the subtitle on an Item.
withSubtitle :: Text -> Item -> Item
withSubtitle s i = i {itemSubtitle = Just s}

-- | Set a single argument on an Item.
withArg :: Text -> Item -> Item
withArg a i = i {itemArg = Just (ArgOne a)}

-- | Set multiple arguments on an Item.
withArgs :: [Text] -> Item -> Item
withArgs as' i = i {itemArg = Just (ArgMany as')}

-- | Add a variable to an Item.
withVar :: Text -> Text -> Item -> Item
withVar k v i = i {itemVariables = Map.insert k v (itemVariables i)}

-- | Remove a variable from an Item.
withUnsetVar :: Text -> Item -> Item
withUnsetVar k i = i {itemVariables = Map.delete k (itemVariables i)}

-- | Set the uid on an Item.
withUid :: Text -> Item -> Item
withUid u i = i {itemUid = Just u}

-- | Set the valid flag on an Item.
withValid :: Bool -> Item -> Item
withValid v i = i {itemValid = Just v}

-- | Set the icon on an Item.
withIcon :: Icon -> Item -> Item
withIcon ic i = i {itemIcon = Just ic}

-- | Set an icon from an image path on an Item.
withIconFromImage :: Text -> Item -> Item
withIconFromImage path i = i {itemIcon = Just (iconFromImage path)}

-- | Set an icon for a filetype on an Item.
withIconForFiletype :: Text -> Item -> Item
withIconForFiletype ft i = i {itemIcon = Just (iconForFiletype ft)}

-- | Add a modifier to an Item. Uses the modifier's modKeys as the map key.
withModifier :: Modifier -> Item -> Item
withModifier m i = i {itemModifiers = Map.insert (modKeys m) m (itemModifiers i)}

-- | Set the autocomplete on an Item.
withAutocomplete :: Text -> Item -> Item
withAutocomplete ac i = i {itemAutocomplete = Just ac}

-- | Set the match field on an Item.
withMatches :: Text -> Item -> Item
withMatches m i = i {itemMatch = Just m}

-- | Set the quicklook URL on an Item.
withQuicklookUrl :: Text -> Item -> Item
withQuicklookUrl url i = i {itemQuicklookUrl = Just url}

-- | Set the copy text on an Item.
withCopyText :: Text -> Item -> Item
withCopyText t i = i {itemText = Just updated}
  where
    existing = maybe (TextContent Nothing Nothing) id (itemText i)
    updated = existing {textCopy = Just t}

-- | Set the large type text on an Item.
withLargeTypeText :: Text -> Item -> Item
withLargeTypeText t i = i {itemText = Just updated}
  where
    existing = maybe (TextContent Nothing Nothing) id (itemText i)
    updated = existing {textLargeType = Just t}

-- | Set the sticky flag on an Item (not serialized to JSON).
withSticky :: Bool -> Item -> Item
withSticky s i = i {itemSticky = s}

-- | Set the boost value on an Item (not serialized to JSON).
withBoost :: Int -> Item -> Item
withBoost b i = i {itemBoost = b}

-- | Slight boost to fuzzy match score (+25 points).
boostSlight :: Int
boostSlight = 25

-- | Low boost to fuzzy match score (+50 points).
boostLow :: Int
boostLow = 50

-- | Moderate boost to fuzzy match score (+75 points).
boostModerate :: Int
boostModerate = 75

-- | High boost to fuzzy match score (+100 points).
boostHigh :: Int
boostHigh = 100

-- | Higher boost to fuzzy match score (+150 points).
boostHigher :: Int
boostHigher = 150

-- | Highest boost to fuzzy match score (+200 points).
boostHighest :: Int
boostHighest = 200
