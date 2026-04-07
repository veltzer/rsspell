# rsspell

A fast and efficient spell checker.

## Design Decisions

### Built-in en-US Dictionary

The `rsspell` tool includes the `en-US` dictionary (`dictionaries/en_US.aff` and `dictionaries/en_US.dic`) embedded directly in the binary. This choice was made to ensure:

1. **Immediate Value:** The tool works out-of-the-box for the most common use case (English spell-checking) without requiring any initial setup or dictionary downloads.
2. **Offline Functionality:** Users can perform spell-checks in English even without an active internet connection.
3. **Robust Fallback:** The embedded dictionary serves as a reliable default if external dictionary management fails or is not used.

### Dictionary Structure (Hunspell Format)

The dictionary is split into two files (`dictionaries/en_US.aff` and `dictionaries/en_US.dic`) following the Hunspell standard. This dual-file approach is used for **affix compression**, which significantly reduces the dictionary's footprint:

*   **`.dic` (Dictionary File):** Contains a list of base words (stems) along with flags that indicate which rules apply to them.
*   **`.aff` (Affix File):** Defines the grammatical rules for prefixes and suffixes (e.g., how to handle plurals, tenses, and possessives).

By separating the rules from the word list, `rsspell` can recognize hundreds of thousands of word variations without needing to store them all explicitly, keeping the binary size small and memory usage efficient.

## Usage

... (rest of the file)
