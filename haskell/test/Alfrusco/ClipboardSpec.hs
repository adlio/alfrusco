{-# LANGUAGE OverloadedStrings #-}

module Alfrusco.ClipboardSpec (spec) where

import Test.Hspec

import Alfrusco.Clipboard (formatHtmlLink, formatMarkdownLink, hexEncode)
import Data.Text.Encoding qualified as TE

spec :: Spec
spec = describe "Alfrusco.Clipboard" $ do
  describe "formatMarkdownLink" $ do
    it "formats basic markdown link" $ do
      formatMarkdownLink "Title" "https://x.com" `shouldBe` "[Title](https://x.com)"

    it "formats with empty strings" $ do
      formatMarkdownLink "" "" `shouldBe` "[]()"

    it "formats with special characters in title" $ do
      formatMarkdownLink "Title [with] brackets" "https://example.com"
        `shouldBe` "[Title [with] brackets](https://example.com)"

    it "formats with query parameters in url" $ do
      formatMarkdownLink "Search" "https://example.com?q=test&p=1"
        `shouldBe` "[Search](https://example.com?q=test&p=1)"

  describe "formatHtmlLink" $ do
    it "formats basic html link" $ do
      formatHtmlLink "Title" "https://x.com" `shouldBe` "<a href=\"https://x.com\">Title</a>"

    it "formats with empty strings" $ do
      formatHtmlLink "" "" `shouldBe` "<a href=\"\"></a>"

    it "formats with special characters" $ do
      formatHtmlLink "Title <with> HTML" "https://example.com"
        `shouldBe` "<a href=\"https://example.com\">Title <with> HTML</a>"

    it "formats with query parameters" $ do
      formatHtmlLink "Search" "https://example.com?q=test&p=1"
        `shouldBe` "<a href=\"https://example.com?q=test&p=1\">Search</a>"

  describe "hexEncode" $ do
    it "encodes ASCII bytes to uppercase hex" $ do
      let bytes = TE.encodeUtf8 "abc"
      hexEncode bytes `shouldBe` "616263"

    it "encodes HTML link to uppercase hex" $ do
      let html = "<a href=\"https://x.com\">Title</a>"
          bytes = TE.encodeUtf8 html
          hex = hexEncode bytes
      -- First character '<' is 0x3C
      take 2 hex `shouldBe` "3C"
