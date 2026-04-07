# Design Decisions

`rsspell` is built with speed and simplicity in mind. This section covers key design choices made during development.

## Built-in en-US Dictionary

The `rsspell` tool includes the `en-US` dictionary (`dictionaries/en_US.aff` and `dictionaries/en_US.dic`) embedded directly in the binary. This choice was made for several reasons:

1. **Immediate Value:** The tool works out-of-the-box for the most common use case (English spell-checking) without requiring any initial setup or dictionary downloads.
2. **Offline Functionality:** Users can perform spell-checks in English even without an active internet connection.
3. **Robust Fallback:** The embedded dictionary serves as a reliable default if external dictionary management fails or is not used.

## Dictionary Structure (Hunspell Format)

The dictionary is split into two files (`dictionaries/en_US.aff` and `dictionaries/en_US.dic`) following the Hunspell standard. This dual-file approach is used for **affix compression**, which significantly reduces the dictionary's footprint:

*   **`.dic` (Dictionary File):** Contains a list of base words (stems) along with flags that indicate which rules apply to them.
*   **`.aff` (Affix File):** Defines the grammatical rules for prefixes and suffixes (e.g., how to handle plurals, tenses, and possessives).

By separating the rules from the word list, `rsspell` can recognize hundreds of thousands of word variations without needing to store them all explicitly, keeping the binary size small and memory usage efficient.

## Supported Formats

`rsspell` currently focuses on file formats where textual content is mixed with structured data.

### Markdown

`rsspell` uses the `pulldown-cmark` library to parse Markdown documents. This ensures that only the text content is checked, while Markdown syntax like links, code blocks, and headers are properly ignored or parsed.

### SVG

For SVG files, `rsspell` parses the XML structure and checks text elements. This is useful for diagrams and icons where typos can often go unnoticed.
