{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.ResponseSpec (spec) where

import Data.Aeson (encode, object, (.=))
import Data.Aeson qualified as Aeson
import Test.Hspec

import Alfrusco.Item (item, withSubtitle)
import Alfrusco.Response

spec :: Spec
spec = describe "Alfrusco.Response" $ do
  describe "empty response" $ do
    it "produces {\"items\":[]}" $ do
      let json = encode defaultResponse
      json `shouldBe` encode (object ["items" .= ([] :: [Aeson.Value])])

  describe "response with items" $ do
    it "includes items in JSON" $ do
      let items = [item "First", withSubtitle "sub" $ item "Second"]
          r = responseWithItems items
          decoded = Aeson.decode (encode r) :: Maybe Aeson.Value
          expected = Just $ object
            [ "items" .= [ object ["title" .= ("First" :: String)]
                         , object ["title" .= ("Second" :: String), "subtitle" .= ("sub" :: String)]
                         ]
            ]
      decoded `shouldBe` expected

  describe "rerun" $ do
    it "serializes rerun as integer when whole number" $ do
      let r = defaultResponse {responseRerun = Just 5}
          decoded = Aeson.decode (encode r) :: Maybe Aeson.Value
          expected = Just $ object
            [ "rerun" .= (5 :: Int)
            , "items" .= ([] :: [Aeson.Value])
            ]
      decoded `shouldBe` expected

  describe "cache" $ do
    it "serializes cache settings" $ do
      let r = defaultResponse {responseCache = Just CacheSettings {cacheSeconds = Just 300, cacheLooseReload = Just True}}
          decoded = Aeson.decode (encode r) :: Maybe Aeson.Value
          expected = Just $ object
            [ "cache" .= object ["seconds" .= (300 :: Int), "loosereload" .= True]
            , "items" .= ([] :: [Aeson.Value])
            ]
      decoded `shouldBe` expected

  describe "skipknowledge" $ do
    it "serializes skipknowledge" $ do
      let r = defaultResponse {responseSkipKnowledge = Just True}
          decoded = Aeson.decode (encode r) :: Maybe Aeson.Value
          expected = Just $ object
            [ "skipknowledge" .= True
            , "items" .= ([] :: [Aeson.Value])
            ]
      decoded `shouldBe` expected

  describe "combined settings" $ do
    it "serializes all settings together" $ do
      let r = defaultResponse
                { responseRerun = Just 3
                , responseCache = Just CacheSettings {cacheSeconds = Just 60, cacheLooseReload = Just True}
                , responseSkipKnowledge = Just True
                , responseItems = [item "Hello"]
                }
          decoded = Aeson.decode (encode r) :: Maybe Aeson.Value
          expected = Just $ object
            [ "rerun" .= (3 :: Int)
            , "cache" .= object ["seconds" .= (60 :: Int), "loosereload" .= True]
            , "skipknowledge" .= True
            , "items" .= [object ["title" .= ("Hello" :: String)]]
            ]
      decoded `shouldBe` expected
