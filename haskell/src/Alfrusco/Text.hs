{-# LANGUAGE OverloadedStrings #-}

-- | Text type for Alfred item copy and large type content.
module Alfrusco.Text
  ( TextContent (..)
  ) where

import Data.Aeson (ToJSON (..), (.=))
import Data.Aeson qualified as Aeson
import Data.Text (Text)

-- | Represents the text options (copy and largetype) for an Alfred item.
-- The copy property is the text copied to the clipboard with CMD-C.
-- The largetype property is displayed when the user presses CMD-L.
data TextContent = TextContent
  { textCopy :: Maybe Text
  , textLargeType :: Maybe Text
  }
  deriving (Show, Eq)

instance ToJSON TextContent where
  toJSON (TextContent copy largeType) =
    Aeson.object $ concat
      [ maybe [] (\c -> ["copy" .= c]) copy
      , maybe [] (\l -> ["largetype" .= l]) largeType
      ]
