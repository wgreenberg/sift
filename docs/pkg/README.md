sift
====

https://wgreenberg.github.io/sift

Command-line/WASM implementation of the excellent qhex wordplay tool
(https://tools.qhex.org/). Useful for cryptic crosswords, puzzle hunts, or any
other time you need to find all 6 letter anagrams that can be reduced to the
word "snarf".

sift supports pipelining transformations, so for example to get all words
that result from transpose-deleting 1 letter from an 8 letter word:

`sift .{8} | sift transpose-delete %` or `sift .{8} | sift td %`

where `%` is replaced w/ each result from `sift .{8}`.

Commands
-----

```
<regex>                      words matching the given regex
add n <letters>              words achievable by adding n letters
anagram <letters>            anagram of the letters
bank <letters>               words using the same set of letters
change n <letters>           words achievable by changing n letters
delete n <letters>           words achievable by deleting n letters
transpose-add n <letters>    words achievable after adding n chars
transpose-delete n <letters> anagram of the letters after deleting n chars
```
