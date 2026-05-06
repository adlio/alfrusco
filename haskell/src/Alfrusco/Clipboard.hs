{-# LANGUAGE OverloadedStrings #-}

-- | Clipboard operations for Alfred workflows.
-- Handles Markdown and Rich Text link copying based on environment variables.
module Alfrusco.Clipboard
  ( handleClipboard
  , formatMarkdownLink
  , formatHtmlLink
  , copyMarkdownLinkToClipboard
  , copyRichTextLinkToClipboard
  , hexEncode
  ) where

import Data.ByteString (ByteString)
import Data.ByteString qualified as BS
import Data.Text (Text)
import Data.Text qualified as Text
import Data.Text.Encoding qualified as TE
import Data.Word (Word8)
import Numeric (showHex)
import System.Environment (lookupEnv)
import System.Exit (ExitCode (..))
import System.IO (stdout)
import System.Process (readCreateProcessWithExitCode, shell)

import Alfrusco.Error (AlfruscoError (..))
import Alfrusco.Response (defaultResponse, writeResponse)

-- | Handle clipboard operations based on environment variables.
-- Returns True if a clipboard operation was performed, False otherwise.
handleClipboard :: IO Bool
handleClipboard = do
  mCmd <- lookupEnv "ALFRUSCO_COMMAND"
  mTitle <- lookupEnv "TITLE"
  mUrl <- lookupEnv "URL"
  case mCmd of
    Just cmd
      | cmd == "richtext" || cmd == "markdown" ->
          case (mTitle, mUrl) of
            (Just title, Just url) -> do
              let titleT = Text.pack title
                  urlT = Text.pack url
              _result <- if cmd == "richtext"
                then copyRichTextLinkToClipboard titleT urlT
                else copyMarkdownLinkToClipboard titleT urlT
              -- Write empty response to stdout regardless of clipboard success
              writeResponse stdout defaultResponse
              pure True
            _ -> pure False
    _ -> pure False

-- | Format a Markdown link: [title](url)
formatMarkdownLink :: Text -> Text -> Text
formatMarkdownLink title url = "[" <> title <> "](" <> url <> ")"

-- | Format an HTML link: <a href="url">title</a>
formatHtmlLink :: Text -> Text -> Text
formatHtmlLink title url = "<a href=\"" <> url <> "\">" <> title <> "</a>"

-- | Copy a Markdown link to the clipboard using pbcopy (macOS) or xclip (Linux).
copyMarkdownLinkToClipboard :: Text -> Text -> IO (Either AlfruscoError ())
copyMarkdownLinkToClipboard title url = do
  let markdown = Text.unpack (formatMarkdownLink title url)
      cmd = "printf '%s' " ++ shellQuote markdown ++ " | pbcopy"
  (exitCode, _, errOutput) <- readCreateProcessWithExitCode (shell cmd) ""
  case exitCode of
    ExitSuccess -> pure (Right ())
    ExitFailure code -> pure (Left (ClipboardError
      ("pbcopy failed with exit code " <> Text.pack (show code) <> ": " <> Text.pack errOutput)))

-- | Copy a Rich Text link to the clipboard using osascript with hex-encoded HTML.
-- Uses AppleScript to set the clipboard to include both plain text and HTML content.
copyRichTextLinkToClipboard :: Text -> Text -> IO (Either AlfruscoError ())
copyRichTextLinkToClipboard title url = do
  let html = formatHtmlLink title url
      hexHtml = hexEncode (TE.encodeUtf8 html)
      -- The AppleScript sets clipboard with both plain text (the URL) and HTML
      script = "set the clipboard to {text:\""
               ++ Text.unpack url
               ++ "\", \171class HTML\187:\171data HTML"
               ++ hexHtml
               ++ "\187}"
      cmd = "osascript -e " ++ shellQuote script
  (exitCode, _, errOutput) <- readCreateProcessWithExitCode (shell cmd) ""
  case exitCode of
    ExitSuccess -> pure (Right ())
    ExitFailure code -> pure (Left (ClipboardError
      ("osascript failed with exit code " <> Text.pack (show code) <> ": " <> Text.pack errOutput)))

-- | Hex-encode a ByteString to uppercase hex characters.
hexEncode :: ByteString -> String
hexEncode = concatMap encodeWord8 . BS.unpack
  where
    encodeWord8 :: Word8 -> String
    encodeWord8 w =
      let h = showHex w ""
      in case h of
           [c]    -> ['0', toUpperChar c]
           [a, b] -> [toUpperChar a, toUpperChar b]
           _      -> h

    toUpperChar :: Char -> Char
    toUpperChar c
      | c >= 'a' && c <= 'f' = toEnum (fromEnum c - 32)
      | otherwise = c

-- | Simple shell quoting: wraps in single quotes and escapes embedded single quotes.
shellQuote :: String -> String
shellQuote s = "'" ++ concatMap escape s ++ "'"
  where
    escape '\'' = "'\\''"
    escape c    = [c]
