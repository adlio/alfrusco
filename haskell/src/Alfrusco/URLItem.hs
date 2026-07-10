{-# LANGUAGE OverloadedStrings #-}

-- | URLItem convenience type for Alfred workflows.
-- Automatically generates clipboard modifiers for Markdown and Rich Text links.
module Alfrusco.URLItem
  ( URLItem (..)
  , urlItem
  , withUrlSubtitle
  , withShortTitle
  , withLongTitle
  , withUrlIcon
  , withUrlIconFromImage
  , withUrlIconForFiletype
  , withDisplayTitle
  , withUrlCopyText
  , withUrlArg
  , withUrlVar
  , toItem
  ) where

import Data.Map.Strict (Map)
import Data.Map.Strict qualified as Map
import Data.Text (Text)

import Alfrusco.Icon (Icon, iconForFiletype, iconFromImage)
import Alfrusco.Item (Item, item, withArg, withCopyText, withIcon, withModifier, withSubtitle, withUid, withValid, withVar)
import Alfrusco.Modifier (Key (..), modifier, modifierCombo, withModArg, withModSubtitle, withModValid, withModVar)

-- | A convenience type for creating URL-based Alfred items with
-- auto-generated clipboard modifiers.
data URLItem = URLItem
  { uiTitle :: Text
  , uiUrl :: Text
  , uiSubtitle :: Maybe Text
  , uiShortTitle :: Maybe Text
  , uiLongTitle :: Maybe Text
  , uiIcon :: Maybe Icon
  , uiDisplayTitle :: Maybe Text
  , uiCopyText :: Maybe Text
  , uiArg :: Maybe Text
  , uiVariables :: Map Text Text
  }
  deriving (Show, Eq)

-- | Create a new URLItem with a title and URL.
urlItem :: Text -> Text -> URLItem
urlItem title url = URLItem
  { uiTitle = title
  , uiUrl = url
  , uiSubtitle = Nothing
  , uiShortTitle = Nothing
  , uiLongTitle = Nothing
  , uiIcon = Nothing
  , uiDisplayTitle = Nothing
  , uiCopyText = Nothing
  , uiArg = Nothing
  , uiVariables = Map.empty
  }

-- | Set the subtitle on a URLItem.
withUrlSubtitle :: Text -> URLItem -> URLItem
withUrlSubtitle s ui = ui {uiSubtitle = Just s}

-- | Set the short title on a URLItem.
withShortTitle :: Text -> URLItem -> URLItem
withShortTitle t ui = ui {uiShortTitle = Just t}

-- | Set the long title on a URLItem.
withLongTitle :: Text -> URLItem -> URLItem
withLongTitle t ui = ui {uiLongTitle = Just t}

-- | Set the icon on a URLItem.
withUrlIcon :: Icon -> URLItem -> URLItem
withUrlIcon ic ui = ui {uiIcon = Just ic}

-- | Set an icon from an image path on a URLItem.
withUrlIconFromImage :: Text -> URLItem -> URLItem
withUrlIconFromImage path ui = ui {uiIcon = Just (iconFromImage path)}

-- | Set an icon for a filetype on a URLItem.
withUrlIconForFiletype :: Text -> URLItem -> URLItem
withUrlIconForFiletype ft ui = ui {uiIcon = Just (iconForFiletype ft)}

-- | Set the display title on a URLItem.
withDisplayTitle :: Text -> URLItem -> URLItem
withDisplayTitle t ui = ui {uiDisplayTitle = Just t}

-- | Set the copy text on a URLItem.
withUrlCopyText :: Text -> URLItem -> URLItem
withUrlCopyText t ui = ui {uiCopyText = Just t}

-- | Set the arg on a URLItem.
withUrlArg :: Text -> URLItem -> URLItem
withUrlArg a ui = ui {uiArg = Just a}

-- | Add a variable to a URLItem.
withUrlVar :: Text -> Text -> URLItem -> URLItem
withUrlVar k v ui = ui {uiVariables = Map.insert k v (uiVariables ui)}

-- | Convert a URLItem to an Item with auto-generated modifiers.
--
-- The conversion:
-- - Sets display title (uiDisplayTitle or uiTitle) as item title
-- - Sets url as subtitle (overridden by uiSubtitle if set)
-- - Sets url as uid
-- - Sets arg (uiArg or url)
-- - Sets url as copy text (overridden by uiCopyText if set)
-- - valid = True
-- - Adds cmd modifier: "Copy Markdown Link '{title}'"
-- - Adds alt modifier: "Copy Rich Text Link '{title}'"
-- - If shortTitle set: adds cmd+shift and alt+shift modifiers
-- - If longTitle set: adds cmd+ctrl and alt+ctrl modifiers
-- - Transfers variables to item
-- - Sets icon if present
toItem :: URLItem -> Item
toItem ui =
  let displayTitle = maybe (uiTitle ui) id (uiDisplayTitle ui)
      title = uiTitle ui
      url = uiUrl ui
      argValue = maybe url id (uiArg ui)

      -- Base cmd modifier: Copy Markdown Link
      cmdMod = withModVar "URL" url
             $ withModVar "TITLE" title
             $ withModVar "ALFRUSCO_COMMAND" "markdown"
             $ withModArg "run"
             $ withModSubtitle ("Copy Markdown Link '" <> title <> "'")
             $ modifier Cmd

      -- Base alt modifier: Copy Rich Text Link
      altMod = withModVar "URL" url
             $ withModVar "TITLE" title
             $ withModVar "ALFRUSCO_COMMAND" "richtext"
             $ withModArg "run"
             $ withModSubtitle ("Copy Rich Text Link '" <> title <> "'")
             $ modifier Alt

      -- Build the base item
      baseItem = withValid True
               $ withCopyText url
               $ withArg argValue
               $ withUid url
               $ withSubtitle url
               $ withModifier cmdMod
               $ withModifier altMod
               $ item displayTitle

      -- Apply custom subtitle override
      withSub = case uiSubtitle ui of
                  Just s -> withSubtitle s
                  Nothing -> id

      -- Apply icon
      withIc = case uiIcon ui of
                 Just ic -> withIcon ic
                 Nothing -> id

      -- Apply short title modifiers
      withShort = case uiShortTitle ui of
                    Just st ->
                      let cmdShiftMod = withModValid True
                                      $ withModVar "URL" url
                                      $ withModVar "TITLE" st
                                      $ withModVar "ALFRUSCO_COMMAND" "markdown"
                                      $ withModArg "run"
                                      $ withModSubtitle ("Copy Markdown Link '" <> st <> "'")
                                      $ modifierCombo [Cmd, Shift]
                          altShiftMod = withModValid True
                                      $ withModVar "URL" url
                                      $ withModVar "TITLE" st
                                      $ withModVar "ALFRUSCO_COMMAND" "richtext"
                                      $ withModArg "run"
                                      $ withModSubtitle ("Copy Rich Text Link '" <> st <> "'")
                                      $ modifierCombo [Alt, Shift]
                      in withModifier cmdShiftMod . withModifier altShiftMod
                    Nothing -> id

      -- Apply long title modifiers
      withLong = case uiLongTitle ui of
                   Just lt ->
                     let cmdCtrlMod = withModValid True
                                    $ withModVar "URL" url
                                    $ withModVar "TITLE" lt
                                    $ withModVar "ALFRUSCO_COMMAND" "markdown"
                                    $ withModArg "run"
                                    $ withModSubtitle ("Copy Markdown Link '" <> lt <> "'")
                                    $ modifierCombo [Cmd, Ctrl]
                         altCtrlMod = withModValid True
                                    $ withModVar "URL" url
                                    $ withModVar "TITLE" lt
                                    $ withModVar "ALFRUSCO_COMMAND" "richtext"
                                    $ withModArg "run"
                                    $ withModSubtitle ("Copy Rich Text Link '" <> lt <> "'")
                                    $ modifierCombo [Alt, Ctrl]
                     in withModifier cmdCtrlMod . withModifier altCtrlMod
                   Nothing -> id

      -- Apply custom copy text override
      withCopy = case uiCopyText ui of
                   Just ct -> withCopyText ct
                   Nothing -> id

      -- Apply custom variables
      withVars = foldr (\(k, v) f -> f . withVar k v) id (Map.toList (uiVariables ui))

  in withVars $ withCopy $ withLong $ withShort $ withIc $ withSub baseItem
