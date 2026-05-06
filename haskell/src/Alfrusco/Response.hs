{-# LANGUAGE OverloadedStrings #-}

-- | Response type representing a complete Alfred workflow response.
module Alfrusco.Response
  ( CacheSettings (..)
  , Response (..)
  , defaultResponse
  , responseWithItems
  , writeResponse
  ) where

import Data.Aeson (ToJSON (..), (.=))
import Data.Aeson qualified as Aeson
import Data.ByteString.Lazy qualified as LBS
import System.IO (Handle)

import Alfrusco.Item (Item)

-- | Cache settings for Alfred 5.5+ cache feature.
data CacheSettings = CacheSettings
  { cacheSeconds :: Maybe Double
  , cacheLooseReload :: Maybe Bool
  }
  deriving (Show, Eq)

instance ToJSON CacheSettings where
  toJSON cs =
    Aeson.object $ concat
      [ maybe [] (\s -> ["seconds" .= renderNumber s]) (cacheSeconds cs)
      , maybe [] (\lr -> ["loosereload" .= lr]) (cacheLooseReload cs)
      ]

-- | Represents a complete Alfred response.
data Response = Response
  { responseRerun :: Maybe Double
  , responseCache :: Maybe CacheSettings
  , responseSkipKnowledge :: Maybe Bool
  , responseItems :: [Item]
  }
  deriving (Show, Eq)

instance ToJSON Response where
  toJSON r =
    Aeson.object $ concat
      [ maybe [] (\rr -> ["rerun" .= renderNumber rr]) (responseRerun r)
      , maybe [] (\c -> ["cache" .= c]) (responseCache r)
      , maybe [] (\sk -> ["skipknowledge" .= sk]) (responseSkipKnowledge r)
      , ["items" .= responseItems r]
      ]

-- | Render a number as an integer if it is a whole number, otherwise as a float.
renderNumber :: Double -> Aeson.Value
renderNumber d
  | d == fromIntegral (round d :: Int) = toJSON (round d :: Int)
  | otherwise = toJSON d

-- | An empty default response with no items.
defaultResponse :: Response
defaultResponse = Response
  { responseRerun = Nothing
  , responseCache = Nothing
  , responseSkipKnowledge = Nothing
  , responseItems = []
  }

-- | Create a response with the given items.
responseWithItems :: [Item] -> Response
responseWithItems items = defaultResponse {responseItems = items}

-- | Write a response as JSON to a Handle.
writeResponse :: Handle -> Response -> IO ()
writeResponse h r = LBS.hPut h (Aeson.encode r)
