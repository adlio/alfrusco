{-# LANGUAGE OverloadedStrings #-}

-- | Fuzzy matching, filtering, and sorting of Alfred items.
module Alfrusco.SortAndFilter
  ( filterAndSortItems
  ) where

import Data.Char (toLower)
import Data.List (partition, sortBy)
import Data.Maybe (mapMaybe)
import Data.Ord (Down (..), comparing)
import Data.Text (Text)
import Data.Text qualified as Text

import Alfrusco.Item (Item (..))

-- | Filter and sort items using fuzzy matching against a query string.
--
-- Behavior:
-- 1. Partition items into sticky and regular.
-- 2. For each regular item, compute a fuzzy match score against "subtitle : title".
-- 3. Add the item's boost to the score.
-- 4. Filter out items with no match.
-- 5. Sort by score descending.
-- 6. Prepend sticky items to the result.
filterAndSortItems :: [Item] -> Text -> [Item]
filterAndSortItems items query =
  let (stickyItems, regularItems) = partition itemSticky items
      scored = mapMaybe scoreItem regularItems
      sorted = sortBy (comparing (Down . snd)) scored
      result = map fst sorted
  in stickyItems ++ result
  where
    queryLower = Text.toLower query

    scoreItem :: Item -> Maybe (Item, Int)
    scoreItem i =
      let subtitle = maybe "" id (itemSubtitle i)
          combined = Text.toLower (subtitle <> " : " <> itemTitle i)
      in case fuzzyScore queryLower combined of
           Nothing -> Nothing
           Just s -> Just (i, s + itemBoost i)

-- | Simple fuzzy matching scorer.
-- Returns Just score if all characters of the query appear in order in the target.
-- Score is based on: exact substring match bonus, consecutive character bonus, etc.
-- Returns Nothing if the query does not match.
fuzzyScore :: Text -> Text -> Maybe Int
fuzzyScore query target
  | Text.null query = Just 0
  | otherwise =
      case fuzzyMatch (Text.unpack query) (Text.unpack target) 0 0 of
        Nothing -> Nothing
        Just score -> Just score

-- | Core fuzzy matching: checks if all characters of the needle appear in order
-- in the haystack, computing a score based on match quality.
fuzzyMatch :: String -> String -> Int -> Int -> Maybe Int
fuzzyMatch [] _ score _ = Just score
fuzzyMatch _ [] _ _ = Nothing
fuzzyMatch needle@(n : ns) (h : hs) score streak
  | toLower n == toLower h =
      let newStreak = streak + 1
          bonus = newStreak * 2  -- consecutive char bonus
      in fuzzyMatch ns hs (score + 10 + bonus) newStreak
  | otherwise = fuzzyMatch needle hs score 0
