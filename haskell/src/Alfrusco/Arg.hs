-- | Arg type for Alfred item arguments.
--
-- An argument can be either a single text value or multiple text values.
-- Serialization is untagged: ArgOne renders as a JSON string, ArgMany as a JSON array.
module Alfrusco.Arg
  ( Arg (..)
  ) where

import Data.Aeson (ToJSON (..), Value (..))
import Data.Aeson qualified as Aeson
import Data.Text (Text)
import Data.Vector qualified as Vector

-- | Represents an argument value for an Alfred item.
-- Can be a single string or an array of strings.
data Arg
  = ArgOne Text
  | ArgMany [Text]
  deriving (Show, Eq)

instance ToJSON Arg where
  toJSON (ArgOne t) = Aeson.String t
  toJSON (ArgMany ts) = Array (Vector.fromList (map Aeson.String ts))
