# CSE 30 (NLP) Prelim Exam Reference Notes

Compiled from course materials for CSE 30, Special Topics 1 (Natural Language Processing and Artificial Neural Networks). This is a reference document for exam preparation, built from lecture slides, the course syllabus, assigned exercises, a paper review exercise, and the Attention Is All You Need paper. It is meant to be read by an LLM later, so it favors completeness and precise technical detail over brevity.

---

## 1. NLP Course Overview

### 1.1 What the course covers

CSE 30 is an elective under BSCS. The course description (from the syllabus) frames NLP as an active area of study under Computing Sciences, with these named subfields: Natural Language Resources, Morphological Analysis, Part of Speech Tagging, Machine Translation, Models for Language Classification, Sentiment Analysis, and the application of NLP in electronic governance. Because current NLP tools may lean on Artificial Neural Network (ANN) fundamentals, the course also covers selected computing fundamentals used in ANNs (McCulloch and Pitts model, the Perceptron, Backpropagation networks).

Course learning outcomes: explain fundamental NLP concepts and theories; create a computer-based solution to a language classification or other NLP-related problem; write a technical paper reporting a solution to an NLP problem; participate effectively in course activities; exhibit academic honesty.

### 1.2 Term-by-term project structure

- Prelim period: domain-based collection and preprocessing of resources for a selected natural language or NLP-related study (example: a Pangasinan or Kapampangan Word-POS-English Translation resource).
- Midterm period: a newly created resource for a selected natural language (example: a Pangasinan lexicon for agriculture, a Kapampangan lexicon for culinary arts).
- Final period: a journal paper presenting and applying an NLP algorithm (classification, correlation, translation, sentiment analysis, etc.), aimed at publication in a journal or presentation at a research conference.

Course goals: build (online) resources for natural languages used in the Cordillera Administrative Region (CAR) and neighboring regions, specifically domain-based electronic lexicons and NLP-based "eGovernance" tools for institutions or social units.

One final course requirement, to be completed by groups, is at least one of:
- A. Present an NLP paper at a conference (submit a conference paper acceptance notice for a presentation happening after the semester).
- B. Publish an NLP paper in a journal (submit a journal paper acceptance notice for publication after the semester).
- C. Deploy an NLP-related application for an actual agency or office (submit the application plus proof of a test run with the client; final deployment may happen later).

### 1.3 Defining NLP

NLP is described as the software or hardware components of a computer system that analyze and synthesize spoken or written language. A commonly cited definition (Konasani & Kadre, 2021): NLP uses Artificial Intelligence to recognize human speech or conversation in any language. A working standard: the language interpretation produced by NLP techniques should be similar to what a human would produce doing the same interpretation.

### 1.4 The two core subsets of NLP (Khurana et al., 2018)

NLP splits into:
1. **Natural Language Understanding (NLU)** – understanding a phrase, sentence, or paragraph. NLU sits under Linguistics and further breaks down into:
   - Phonology
   - Morphology
   - Syntax
   - Semantics
   - Pragmatics
   (Discourse is also listed as a related terminology alongside these.)
2. **Natural Language Generation (NLG)** – producing a phrase, sentence, or paragraph. NLG's output is natural language text.

A current approach layered on top of this framework is **Retrieval Augmented Generation (RAG)**.

### 1.5 NLP concern examples shown in the course slides

- Morphological analysis (lemmatization, stemming): breaking "pag-iibigan" into "pag-", "i-", "ibig", "-an".
- Machine translation: "Umali Kayo." to "You are welcome here." / "You come here."
- Sentiment analysis: "Food is tasty and affordable at PCC" to Positive Sentiment, "You should visit the restaurant."
- Entity recognition: extracting "14 February 2023" as a date entity from a longer sentence.

### 1.6 Core NLP tasks (non-exhaustive list from the course slides)

- Automatic Summarization: produces an understandable summary of a set of text, providing summaries or detailed information of a known text type.
- Co-Reference Resolution: determines which words in a sentence or larger text refer to the same object.
- Discourse Analysis: identifies the discourse structure of connected text.
- Machine Translation: automatic translation of text from one human language to another.
- Morphological Segmentation: separating a word into individual morphemes and identifying the class of each morpheme (relates to stemming and lemmatization).
- Named Entity Recognition (NER): given a stream of text, determines which items relate to proper names.
- Optical Character Recognition (OCR): given an image representing printed text, determines the corresponding text.
- Part of Speech Tagging: given a sentence, determines the part of speech for each word.

These tasks are closely interwoven; some (like automatic summarization or co-reference analysis) act as subtasks inside larger tasks.

### 1.7 NLP spectrum (as pictured in the slides)

Corpus Building -> Morphological Analysis -> Segmentation -> Tokenization -> Stemming -> POS tagging -> Language Translation -> Sentiment Analysis -> Topic Modeling -> (and further tasks beyond this).

### 1.8 Terminologies in NLP (linguistics levels)

Phonology, Morphology, Lexical Level, Syntactic Level, Semantic Level, Discourse, Pragmatic Level.

### 1.9 Applications of NLP

Machine Translation, Text Categorization, Spam Filtering, Information Extraction, Summarization, Dialog Systems, Medicine (Specialist Systems).

### 1.10 Areas of study under NLP (as organized for this course)

- Sentiment Analysis
- Topic Modeling
- Morphological Analysis (electronic lexicon, morphology, stemming)
- Part of Speech Tagging (tagset; rule-based and probabilistic approaches)
- Machine Translation (unidirectional; bidirectional)

### 1.11 Morphology and stemming, high level

- Derivation morphology
- Inflection morphology, with these mechanisms:
  - Affixation
  - Reduplication
  - Agglutination

(Full detail on morphology is in Section 5.)

### 1.12 NLP and social media / governance framing

A recurring course theme: "making sense out of social media activity" includes extracting information, recognizing talents, rewarding good deeds, and exposing corruption. The slides note (as an observed phenomenon, not a research claim) that "unpleasant social media posts are more appreciated."

### 1.13 Course references worth knowing by name

- Konasani, V. & Kadre, S. (2021). *Machine Learning and Deep Learning Using Python and TensorFlow.* McGraw-Hill.
- Khurana, D. et al. (2018). *Natural Language Processing: State of the Art, Current Trends and Challenges.*
- Brill, E. (2000). *Part-of-Speech Tagging.* In *Handbook of Natural Language Processing.*
- Flordeliza, J., Go, K., & Miguel, D. (2005). *PTPOST4.0: Probabilistic Tagalog Part of Speech Tagging.* De La Salle University.
- See, S. C. (2006). *Tagalog Morphological Analyzer Using an Example-Based Approach.* MSCS Thesis, De La Salle University.
- Wicentowski, R. (2002). *Modeling and Learning Multilingual Inflectional Morphology in a Minimally Supervised Framework.* Johns Hopkins University.

---

## 2. Part of Speech (POS) Tagging

### 2.1 Definition and purpose

POS tagging means assigning the correct part of speech (noun, verb, adjective, etc.) to each word in a sentence. Example given in the slides:

- Sentence: "Siya ay inaantok."
- Tags: PRS LM JJD PMP

Relevance of POS tagging:
- Sentence parsing, for syntactically correct translation.
- Word sense disambiguation, for semantically correct translation.
- Word pronunciation, for audio-focused applications (text-to-speech).

### 2.2 Tagsets

A **tagset** is a collection of possible tags used to annotate a corpus.

Tagalog tagsets:
- Tagalog tagset by Rabo (2004): 59 tags.
- Revised Tagalog tagset by Dr. Buban: 65 tags.
- The course leaves "YOUR tagset" open as a design choice for the student's own language project.

English-based tagsets:
- **Penn Treebank Tagset**: example tags include NN, NNS, NNP, NNPS, PRP, PRP$, WP, WP$, VB, VBD, JJ, JJR, JJS, RB, etc.
- **CLAWS Tagset**: example tags include NN1, NN2, NPO, PNI, PNP, VVO, VVD, AJO, AJC, AJS, AVO, AVQ, AT, PRP, PUN, etc. (Different CLAWS versions have evolved over time.)
- **Universal Tagset**: a cross-language tagset, referenced but not enumerated in detail in the slides.

Corpora annotation is performed by a linguist: raw corpus words are matched against a tagset to produce an annotated corpus (word-tag pairs). From the annotated corpus, an NLP system induces a lexicon and rules database.

### 2.3 POS tagging approaches

The overall taxonomy (as diagrammed in the slides):

```
POS Tagging
├── supervised
│   ├── rule-based
│   ├── stochastic
│   │   └── maximum likelihood
│   │       └── n-grams
│   │           └── Hidden Markov Model, Viterbi Algorithm
│   └── neural
└── unsupervised
    ├── rule-based
    ├── stochastic
    │   └── Baum-Welch
    └── neural
```

#### Rule-based POS tagging

Uses a database of words and rules. Example rule: "a word preceded by a determiner and followed by a noun is an adjective."

#### Probabilistic (stochastic) POS tagging

Uses probability concepts embedded in abstract machines and algorithms. The goal is to find a sequence of tags T = {t1, t2, t3, ..., tn} that is optimal for a word sequence W = {w1, w2, w3, ..., wn}, using:

**P(T|W) = P(T) * P(W|T) / P(W)**

Key sub-concepts:
- The Markov Assumption, realized as bigram, trigram, or general n-gram models.
- Lexical probability.
- Contextual probability.

A worked example (a "PTPOST" style model, using a simple Tagalog sentence structure) shows a start state S branching into JJD (adjective), DTC (determiner), and PRS (pronoun) paths with associated probabilities (e.g., 0.25 for "Maliwanag", 0.25 for "Ang", 0.50 for "Ako"/"Siya"), each path then continuing deterministically (probability 1.0) into the next tag and word (e.g., DTC -> NNC with word "tao"; PRS -> LM with word "ay").

Supervised tagging procedure:
1. Select a tagset / tagged corpus.
2. Create dictionaries using the tagged corpus.
3. Calculate disambiguation tools: word frequencies, affix frequencies, tag sequence probabilities, "formulaic" expressions.
4. Tag test data using the dictionary information and the calculated disambiguation tools.
5. Calculate tagger accuracy.

Unsupervised tagging procedure (same general shape, but starting from untagged data):
1. Induce a tag set from untagged training data.
2. Induce a dictionary from the training data.
3. Calculate the same disambiguation tools (word frequencies, affix frequencies, tag sequence probabilities, formulaic expressions).
4. Tag test data using the induced dictionaries and calculated disambiguation tools.
5. Calculate tagger accuracy.

### 2.4 Preparing POS tagging data (pipeline)

1. Start with a Text File and a Tag File.
2. Check whether all tags in the Tag File are valid; if invalid tags exist, edit the tag file and recheck.
3. Match words to tags. If the number of words does not equal the number of tags, edit the tag file or the text file and re-match.
4. Once counts match, subdivide the text and tag files into training and test data: Train Text File, Train Tag File, Test Text File, Test Tag File.

### 2.5 Framework for analyzing POS taggers

Training data (Text File + Tag File) feeds a **POS Training** process that writes to shared Databases/Tables and produces a Training Time Record. Test data (Text File) feeds a **POS Tagging** process (using the same Databases/Tables) that produces Generated Word-Tag pairs and an Execution Time Record. A **Tag Comparison** step compares the generated word-tag pairs against the Correct Tags (Test Data), and the comparison plus the timing records feed into an **Analysis and Performance Statistics** step that outputs Summary Tables.

### 2.6 Evaluation and performance metrics

General NLP evaluation metrics named in the course: **Accuracy**, **Precision**, **Recall**.

POS-tagging-specific performance metrics:
- **Tagging Accuracy**: number of correctly tagged words divided by the number of words subjected to tagging.
- **Tagging Time per word**: total time to tag a corpus divided by the number of words in the corpus.
- **Error rate**: the complement of accuracy.
- **Accuracy on Unknown Words**: accuracy measured specifically on words the tagger had not seen during training.

### 2.7 Observation points when studying an annotated corpus

- Factors affecting the validity of annotated corpora.
- Frequency distribution of tags on an annotated corpus.
- Wrong tags distribution.
- Context of wrong tags.
- Unknown words distribution.

Factors affecting corpus validity, specifically around sentence delimiters:
- Sentence delimiters: period, question mark, exclamation point, plus edge cases like double quotes and ellipses.
- Abbreviations with periods: the tagging process must distinguish a period that ends an abbreviation from a period that ends a sentence.

Example observations (from a Tagalog-tagged corpus case study in the course):
- Highest-frequency single tag: NNC (common noun).
- Most frequent tags overall: NNC, PMP, DTC, NNP, CCT, CCB.
- Least-frequency tag (occurring zero times in that corpus): VBOI.
- Least frequent tags generally: VBOI, PRF, VBTP, JJCN, PRQP, TS, VBOL, RBJ.
- Tags most often assigned to unknown words, in descending order of occurrence: NNC, VBTF, VBTR, NNP, JJD, VBW, VBTS, NNPA, JJCC, RBD, VBOF.

### 2.8 Worked exercise: POS tagging with Python/NLTK

The August 20, 2026 exercise asked for a short Python program illustrating POS tagging on at least 10 words, using the Natural Language Toolkit (NLTK). The submitted solution:

```python
import nltk

nltk.download("averaged_perceptron_tagger_eng", quiet=True)
nltk.download("punkt_tab", quiet=True)

sentence = "The curious cat quickly climbed the tall tree and happily watched three birds ..."

tokens = nltk.word_tokenize(sentence)
tagged = nltk.pos_tag(tokens)

for word, tag in tagged[:10]:
    print(f"{word:12} -> {tag}")
```

Output (first 10 tokens, using the Penn Treebank tagset since that is NLTK's default `pos_tag` tagset):

```
The       -> DT
curious   -> JJ
cat       -> NN
quickly   -> RB
climbed   -> VBD
the       -> DT
tall      -> JJ
tree      -> NN
and       -> CC
happily   -> RB
```

The approach: tokenize the sentence into words, then run NLTK's tagger over the whole token sequence so each word is tagged using its actual context rather than guessed in isolation. The 10 words demonstrate determiners (DT), adjectives (JJ), nouns (NN), adverbs (RB), a past-tense verb (VBD), and a coordinating conjunction (CC).

---

## 3. Bag of Words (BoW)

### 3.1 Concept

Bag of Words is a way to turn text into a numeric vector for a fixed vocabulary. Each position in the vector corresponds to one vocabulary word (by index), and the value at that position is the count of how many times that word appears in the given text. A BoW vector records **which words appear and how many times**, but **not the order** they appeared in. This means a single vector is consistent with many different word orderings of the same multiset of words; you cannot recover word order from the vector alone.

### 3.2 Worked exercise (September 10, 2026)

The exercise vocabulary had 44 Tagalog words, indexed 1 (aking) through 44 (Tutuparin), derived by lowercasing a source text (the Panatang Makabayan, the Filipino patriotic pledge).

**Item 1 – decoding a sparse vector.** Given a vector with 1s at positions 3 (ang), 11 (iniibig), 15 (ko), and 36 (Pilipinas) and 0s elsewhere, the words present are ang, iniibig, ko, Pilipinas. Recovering the actual phrase "Iniibig ko ang Pilipinas" (the opening line of the Panatang Makabayan) required outside knowledge of that source text; the vector by itself permits any ordering of those four words, which is exactly the information BoW discards.

**Item 2 – building the full-text vector.** For the complete 64-word pledge text, the vector has a 1 for each vocabulary word that appears exactly once, and a higher count for repeated words. In this exercise, six words repeated: aking (4), ang (6), at (3), ko (6), ng (4), Pilipinas (3); the other 38 vocabulary words each appeared once. All counts sum to 64, matching the total word count of the text.

**Item 3 – effect of stopword removal.** Removing stopwords {ang, at, ko, mga, nang, ng, sa} deletes their vocabulary positions and shifts everything after them left, shrinking the vocabulary from 44 words to 37 (renumbered 1 to 37, alphabetically). The seven removed stopwords had accounted for 22 of the 64 total word occurrences (6+3+6+1+1+4+1), so the new vector's counts sum to 42. Crucially, the counts for surviving words do not change; only their positions shift because of the deleted columns. In the new 37-slot vector, aking keeps count 4 at position 1, and Pilipinas keeps count 3, now at position 30 (moved down from its old position 36 because six stopword slots before it were deleted).

**Item 4 – decoding a second, post-stopword-removal vector.** A 37-length vector with a 1 at position 9 (iniibig) and position 30 (Pilipinas) and 0s elsewhere corresponds to the same opening line "Iniibig ko ang Pilipinas," but with the stopwords ko and ang already stripped, leaving only iniibig and Pilipinas visible. Even though Pilipinas appears 3 times across the whole text, the count here is 1, confirming the vector describes only this one line, not the full document.

### 3.3 Takeaways for the exam

- BoW vector length equals vocabulary size, not text length.
- The sum of a BoW vector's entries equals the number of (non-stopword-filtered) word occurrences in the corresponding text.
- Removing stopwords changes vocabulary size and renumbers indices, but never changes the raw counts of surviving words.
- Decoding a BoW vector back into readable text requires either knowing the source text or accepting that word order is fundamentally ambiguous.

---

## 4. Morphology

Morphology is the study of word structure. It sits at the conceptual center of linguistics because a word is the interface between phonology, syntax, and semantics: words have phonological properties, articulate into phrases and sentences, their forms reflect syntactic function, and their parts are often composed of smaller, meaningful pieces. Words also relate to each other by form, forming paradigms and lexical groupings.

Two central questions morphology addresses:
1. What governs morphological form?
2. What governs the syntactic and semantic function of morphological units, and how does that interact with syntax and semantics?

### 4.1 Core vocabulary

- **Morpheme**: the basic building block of words (prefixes, suffixes, roots). Divided into **lexical morphemes** and **functional morphemes**.
- **Lexeme**: a unit of linguistic analysis belonging to a particular syntactic category, with a particular meaning or grammatical function, that enters syntactic combinations as a single word.
- **Paradigm**: the full set of words realizing a particular lexeme. Example: {cantare' (root form used illustratively), sing, sang, sung, sings, singing, ...} all realize the "sing" lexeme.
- **Root** (a lexeme's root): a unit of form from which a paradigm of phonological words is deduced.

### 4.2 Morphological phenomena (general list)

Derivation, Inflection, Incorporation, Clitic, Compounding, Infixation, Suffixation, Prefixation, Circumfixation, Reduplication (and others).

- **Inflection**: generating a word of the *same* part of speech as its root. Example: fear -> fears (present tense, singular subject); talk -> talked (past tense). Both members of each pair are verbs.
- **Derivation**: generating a word with a *different* meaning or classification from its root. Example: malice (noun) -> malicious (adjective).
- **Closure Principle in Morphology**: inflection closes a word off from further derivation, while derivation does not.
- **Incorporation**: concatenating a word (such as a verb) with another word (noun, pronoun, or adverb) to realize a combined syntactic function. Kankanaey example: "kinanko" = "kinan" (ate) + "ko" (I), corresponding to the English "I ate it." Another example: "edwani" ("in the present time") comes from "ed" (in) + "nuwani" (present), with the "nu" syllable of "nuwani" disappearing on fusion, which looks like a clitic phenomenon.
- **Clitic (enclitic)**: a reduced form fused onto a host word. English examples: 'm in I'm, 's in he's, n't in don't/can't. The Kankanaey "edwani" example above also illustrates a clitic-like disappearance of a syllable.
- **Compounding**: two or more words sequenced together correspond to a single meaning. English example: "Green House." Kankanaey example: from root "talak" (car), "taltalak" means "toy car."
- Additional phenomena catalogued for natural languages generally: **Infixation** (applying infixes to root stems), **Prefixation** (applying prefixes to root stems), **Suffixation** (applying suffixes to root words), **Partial Reduplication** (repeating some letters or syllables of a root stem), **Full Reduplication** (repeating the whole root stem), **Replacive Affixation** (replacing, inserting, or deleting a letter in the root stem).
- Six morphological phenomena posited by Wicentowski (2002) that can apply across languages to produce derivations and inflections: **Simple Affixation, Vowel Harmony, Internal Vowel Shift, Agglutination, Reduplication, Template Filling**. Wicentowski found simple affixation, vowel harmony, internal vowel shift, and reduplication to be true for Tagalog.
  - **Vowel Harmony**: alteration of a root word's phonological content due to affixation, following vowel-harmony preferences. English illustration: sweep -> swept.
  - **Internal Vowel Shift**: systematic vowel change in a root/inflection pair that is *not* due to an affix.
  - **Agglutination**: concatenation of multiple affixes when deriving an inflection.
  - **Template Filling**: applying patterns of affixation, vowel insertion, or phonological change to produce an inflection.

Many Philippine languages are enriched by derivation and inflection.

### 4.3 Kankanaey morphology case study: word families from a single root (course slide example, Miguel 2009)

In Kankanaey, nouns and verbs are repositories of many related word forms. The example root "gabyon" (a hand-operated hoe used in vegetable farming) generates roughly 50 related words across categories:

- Noun: gabyon (hoe).
- Verbs: gabyonan (to use a hoe to dig a portion of land), gabyonen (will use a hoe to dig an object), ginmabyon (did the act of using a hoe), gumabyon (will do the act of using a hoe, speaker is the object), mangabyon (to use / will use a hoe), nangabyon (used a hoe), and many more inflected verb forms marking tense, aspect, and whether the action is done alone or alongside other actors (e.g., makigabyon = will use a hoe alongside other actors; nakigabyon = used a hoe alongside other actors).
- Adjectives: ginabgabyon (referring to an object for which a hoe had been used), ginabyon (referring to an object for which a hoe was used), pangabyon (referring to an instrument for digging), magabyon (referring to a person capable of using a hoe efficiently), makagabyon (referring to the eagerness to use a hoe), and others describing places, reasons, or objects connected with using the hoe.
- Compounds/Incorporations built from the same root: gabgabyon (small hoe / toy hoe), mangabyonda (they will use a hoe, verb+pronoun fusion), nangabyonak (I used a hoe), pangabyonko (hoe for me to use, noun+verb+pronoun fusion), gabyonda (their hoe, noun+pronoun fusion), gabyonko (my hoe), and equivalent forms for other pronoun persons/numbers.

A second example root, "ali" (to come), likewise generates a large paradigm: inmali (came, simple past), inmalali (had always been coming, past continuous), umali (comes/will come), umaliak (I come/I will come), umalida (they will come), makiali (will come as a companion), nakiali (came as a companion), maki-al-ali (coming along with other objects), and adjective forms like umalian (referring to the time of coming) and kaanali (referring to objects who suddenly came).

Observations drawn from these two case studies: infixation, prefixation, suffixation, reduplication, and replacive affixation are all real phenomena for Kankanaey (and, by extension, plausibly other Philippine languages); the applicable infixes, prefixes, and suffixes can be defined as formal sets; and regular rules supporting these morphological phenomena can be formulated as functions. This motivates the more detailed functional modeling in the Kankanaey adjectives study below (Section 4.4).

### 4.4 Kankanaey adjective inflections (Miguel, "Modeling Kankanaey Adjective Inflections")

This is a dedicated research report on Kankanaey, an indigenous language spoken in Benguet and the wider Cordillera Administrative Region (CAR), Philippines. About 1500 root words were collected from native speakers (through questionnaires, observed formal and informal speech, and scanned written materials such as song lyrics and Biblical transcripts); of these, more than 700 are nouns, more than 200 are adjectives, more than 400 are verbs, plus smaller counts of pronouns, adverbs, conjunctions, and prepositions.

Background terms used in the paper (aligned with Wicentowski's morphology framework):
- **Inflection**: generates a word of the same part of speech as the base. Tagalog example: sulat (to write, infinitive) -> sumulat (wrote, past tense); both are verbs.
- **Derivation**: generates a word of a different part of speech than the base. English example: mountain (noun) -> mountainous (adjective).
- Affixation phenomena illustrated in Tagalog: Prefixation (iyak -> umiyak), Infixation (punta -> pumunta), Suffixation (palit -> palitan), Circumfixation (mahal -> nagmahalan).

Basic Kankanaey phonology assumptions used in the study (no formal orthography exists for the language, so these are heuristic): pronounce words as spelled; two consecutive vowels form two separate syllables (e.g., "pao" is read "pa-o"); the letter "i" is pronounced as in the English word "kid"; the letter "e" is pronounced as the first "e" in the English word "were."

#### 4.4.1 Classifying Kankanaey adjectives

Kankanaey adjectives are either **adjectives by nature** (the adjective meaning is carried by an unanalyzed root word, e.g., ando = long, aptik = short, dakdake = big) or **derived adjectives** (composed of a root word plus an affix, where the root itself may belong to a different part of speech, e.g., mabikas = strong, derived from the noun bikas = strength; nalaing = intelligent, derived from the noun laing = intelligence).

#### 4.4.2 Comparative adjective formation

For adjectives by nature: reduplicate the first syllable of the root. If that first syllable is a vowel+consonant (VC) sequence, insert a hyphen between the reduplicated syllable and the base word; if it is a consonant+vowel+consonant (CVC) sequence, no hyphen is inserted. Examples: aptik (short) -> ap-aptik (shorter); dakdake (big) -> dakdakdake (bigger); ando (long) -> an-ando (longer).

Formally: f: X -> Y where f(x) = y, X is the set of adjectives by nature, Y is the set of comparative adjectives, and y is the concatenation of (the first syllable of x, a hyphen if that syllable is VC, and x), or the concatenation of (the first syllable of x and x) if that syllable is CVC.

For derived adjectives: reduplicate the first syllable of the *root word* the adjective was derived from (not the derived adjective's own first syllable), then reattach the original derivational prefix. Example: mabikas (strong) comes from bikas (strength); reduplicating "bik" gives mabikbikas (stronger). Example: nalaing (intelligent) comes from laing (intelligence); reduplicating "lal" gives nalallaing (more intelligent).

Formally: g: A -> Y where g(a) = y, A is the set of derived adjectives, and y is the concatenation of (the derivational prefix used when a was derived, the first syllable of a, a hyphen if that syllable is VC, and a) or (the prefix, the first syllable of a, and a) if that syllable is CVC.

Note: some speakers use an alternate comparative form for derived adjectives that instead reduplicates the derived word's own first syllable (e.g., mabmabikas instead of mabikbikas; nalnalaing instead of nalalaing). The paper recommends supporting both inflection processes together to capture natural variation.

#### 4.4.3 Superlative adjective formation

For adjectives by nature: circumfix "ka-" and "-an" around the base adjective (ka prefixed, an suffixed). Examples: ando (long) -> kaandoan (longest); kitkitoy (small) -> kakitkitoyan (smallest).

Formally: h: X -> Z where h(x) = z, and z is the concatenation of "ka", the base adjective x, and "an".

For derived adjectives: circumfix "ka-" and "-an" around the *root word* (not the derived adjective) that the adjective was derived from. Examples: mabikas (strong, from bikas = strength) -> kabikasan (strongest); maapat (talkative, from apat = talk) -> kaapatan (most talkative).

Formally: k: A -> Z where k(a) = z, and z is the concatenation of "ka", the root word of derived adjective a, and "an".

#### 4.4.4 Adverbial numeral adjectives ("only" / "just")

Repeating the first syllable of a Kankanaey number term produces the meaning "only [number]" or "just [number]." Examples: esa (one) -> es-esa (only one/just one); duwa (two) -> duduwa (only two); simpu (ten) -> simsimpu (only ten); singgasot (one hundred) -> singsinggasot (only one hundred); sinlibo (one thousand) -> sinsinlibo (only one thousand).

Formally: m: V -> W where m(v) = w, V is the set of Kankanaey numeral adjectives, and w is the concatenation of (the first syllable of v, a hyphen if VC, and v) or (the first syllable of v and v) if CVC.

#### 4.4.5 Adverbial adjectives ("very")

Full word reduplication of a root adjective that ends in a vowel produces the meaning "very [adjective]." Example: ando (long) -> andoando (very long).

For adjectives ending in a consonant, reduplication is partial: the ending consonant is excluded from the reduplicated part. Example: aptik (short) -> aptiaptik (very short), not "aptikaptik."

For a derived adjective, the reduplication is performed on the *root word*. Example: nalaing (intelligent, root laing) -> nalailaing (very intelligent). If the root word has more than two syllables, only the first two syllables are reduplicated: asag-en (near) -> asa-asag-en (very near); adawi (far) -> adaadawi (very far).

Formally: j: O -> P where j(o) = p, O is the set of Kankanaey adjectives, and p is the concatenation of (the adjective's first two syllables and the adjective itself) if o is an adjective by nature, or (the derivational prefix used, the first two syllables of the root word, and o itself) if o is a derived adjective.

#### 4.4.6 Exceptions and irregularities

Not every adjective follows these rules. Kankanaey manmano (rare) and namaga (lost) do not undergo the standard comparative/superlative formation. **Replacive affixation** appears in some cases: namasig (insistent, from root pasig = insistence) forms the comparative namaspasig (more insistent) rather than the expected namasmasig, because the rule maintains the original letter on the second occurrence of a reduplicated syllable. (Some native speakers do still accept namasmasig as meaning "more insistent.")

#### 4.4.7 Cross-language comparison

- **Ibaloi** (another Benguet indigenous language): also uses ka-an circumfixation for superlatives, e.g., nabanol (valuable) -> kabanolan (very valuable); mapteng (beautiful) -> mamapteng (more beautiful).
- **Ilokano** (the CAR's regional language): very similar affixation/reduplication patterns to Kankanaey. bassit (small) -> basbassit (smaller) -> kabassitan (smallest). nalukneng (soft, derived) -> naluklukneng (softer) -> kaluknengan (softest). maysa (one) -> maymaysa (only one). ado (many) -> adoado (very many).
- **Tagalog**: simpler construction by comparison. Comparative adjectives prefix the base with "mas"; superlative adjectives prefix the base with "pinaka". Example: malinaw (clear) -> masmalinaw (more clear) -> pinakamalinaw (most clear). The paper notes that some Tagalog-acculturated modern Kankanaey speakers have started adopting this simpler Tagalog pattern in their own speech.

#### 4.4.8 Summary conclusions

1. Reduplicating the first syllable of a Kankanaey adjective-by-nature generates its comparative form.
2. Reduplicating the first syllable of a derived adjective's root word generates its comparative form.
3. Circumfixing ka-/-an on an adjective-by-nature generates its superlative form.
4. Circumfixing ka-/-an on a derived adjective's root word generates its superlative form.
5. Partial reduplication of numeral adjectives generates the "only/just [number]" adverbial numeral adjective.
6. Full reduplication of vowel-ending root adjectives generates the "very [adjective]" adverbial adjective.
7. Partial reduplication of consonant-ending root adjectives (excluding the final consonant) generates the "very [adjective]" adverbial adjective.

These affixation and reduplication processes can all be modeled as mathematical functions from a set of base adjectives to a set of comparative, superlative, adverbial-numeral, or adverbial adjectives, and this function-based framework is intended to support building an electronic lexicon or word-generation tool for Kankanaey.

---

## 5. Naive Bayes for Language Data

The syllabus lists "Language Model for Classification" and the "Naive-Bayes Formula" as Prelim-period content, taught through a workshop on classifying or identifying which natural language a given piece of text belongs to. The accompanying raw data file for this topic (`NaiveBayes_LanguageData.xlsx`) is a spreadsheet of language-classification training data and was intentionally not read in detail for this summary (raw data, not conceptual content), so only the general framework is captured here.

General Naive Bayes classification framework, as applied to language/text classification: given a document or word to classify into one of several classes (for example, into different languages, or into positive/negative/neutral sentiment), Naive Bayes estimates the class C that maximizes:

**P(C | features) proportional to P(C) * P(features | C)**

where P(features | C) is computed by naively assuming the features (e.g., individual words or character n-grams) are conditionally independent given the class, so P(features | C) = product over each feature f of P(f | C). For language identification specifically, the "features" are typically the words or character sequences observed in the text, and P(word | Language) is estimated from word-frequency counts within a labeled corpus for each candidate language. The predicted language is the one whose class-conditional probability, combined with the prior probability of that language, is highest.

This is conceptually the same probabilistic reasoning shown for POS tagging in Section 2.3 (P(T|W) = P(T) * P(W|T) / P(W)), generalized from tag sequences to whole-text classification.

---

## 6. Attention Is All You Need (the Transformer)

Source: Vaswani et al., "Attention Is All You Need," Google Brain / Google Research, NIPS 2017 (arXiv:1706.03762). This is the paper that introduced the Transformer architecture, the foundation for BERT and other modern large language models referenced elsewhere in this course.

### 6.1 Core claim

Prior state-of-the-art sequence transduction models (for tasks like machine translation) used recurrent neural networks (RNNs), including LSTMs and gated recurrent units, in an encoder-decoder structure, often connected by an attention mechanism. Recurrent models are inherently sequential: they compute a hidden state h_t as a function of the previous hidden state h_(t-1) and the input at position t, which prevents parallelization within a training example and becomes a bottleneck at long sequence lengths. Convolutional alternatives (Extended Neural GPU, ByteNet, ConvS2S) compute hidden representations in parallel, but the number of operations needed to relate two arbitrary positions still grows with the distance between them (linearly for ConvS2S, logarithmically for ByteNet).

The Transformer is presented as the first transduction model that relies **entirely on self-attention**, with no recurrence and no convolution, to compute representations of its input and output. In the Transformer, the number of operations required to relate signals between any two positions is reduced to a constant (O(1)), at the cost of reduced effective resolution from averaging attention-weighted positions; this resolution loss is counteracted with Multi-Head Attention.

### 6.2 Overall architecture

The Transformer keeps the encoder-decoder structure common to competitive sequence transduction models: the encoder maps an input sequence of symbol representations (x1, ..., xn) to a sequence of continuous representations z = (z1, ..., zn); given z, the decoder generates an output sequence (y1, ..., ym) one element at a time, auto-regressively (consuming previously generated symbols as additional input when generating the next one).

**Encoder**: a stack of N = 6 identical layers. Each layer has two sub-layers:
1. A multi-head self-attention mechanism.
2. A simple, position-wise, fully connected feed-forward network.

A residual connection is applied around each of the two sub-layers, followed by layer normalization. That is, each sub-layer's output is `LayerNorm(x + Sublayer(x))`. To make the residual connections work, every sub-layer and every embedding layer produces outputs of dimension d_model = 512.

**Decoder**: also a stack of N = 6 identical layers. In addition to the encoder's two sub-layer types, the decoder inserts a third sub-layer that performs multi-head attention over the output of the encoder stack. Residual connections and layer normalization are applied the same way. The decoder's self-attention sub-layer is modified (masked) to prevent positions from attending to subsequent (future) positions; combined with offsetting the output embeddings by one position, this masking ensures that the prediction for position i can depend only on known outputs at positions less than i (this preserves the auto-regressive property).

### 6.3 Attention

An attention function maps a query and a set of key-value pairs to an output, where the query, keys, values, and output are all vectors. The output is a weighted sum of the values, where each value's weight comes from a compatibility function between the query and the corresponding key.

#### 6.3.1 Scaled Dot-Product Attention

Inputs: queries and keys of dimension d_k, values of dimension d_v. Compute the dot product of the query with every key, divide each result by sqrt(d_k) (the scaling factor), then apply softmax to obtain the weights on the values.

In matrix form, packing queries into Q, keys into K, and values into V:

**Attention(Q, K, V) = softmax( (Q K^T) / sqrt(d_k) ) V**

This is compared against the two most common attention functions: **additive attention** (Bahdanau et al.), which computes compatibility with a feed-forward network with one hidden layer, and **dot-product (multiplicative) attention**, which the paper's method matches except for the 1/sqrt(d_k) scaling factor. Dot-product attention is much faster and more space-efficient in practice because it can be implemented with highly optimized matrix multiplication. For small d_k the two mechanisms perform similarly, but for large d_k, additive attention outperforms unscaled dot-product attention; the paper's hypothesis is that for large d_k, dot products grow large in magnitude (since if q and k components are independent random variables with mean 0 and variance 1, their dot product has mean 0 and variance d_k), pushing softmax into regions with extremely small gradients. Scaling by 1/sqrt(d_k) counteracts this.

#### 6.3.2 Multi-Head Attention

Rather than performing a single attention function using the full d_model-dimensional keys, queries, and values, the queries, keys, and values are linearly projected h times using different learned linear projections, down to dimensions d_k, d_k, and d_v respectively. Attention is then computed in parallel on each of these h projected versions, yielding h outputs of dimension d_v each. These are concatenated and projected once more to produce the final output.

**MultiHead(Q, K, V) = Concat(head_1, ..., head_h) W^O**

**where head_i = Attention(Q W_i^Q, K W_i^K, V W_i^V)**

Here W_i^Q and W_i^K are in R^(d_model x d_k), W_i^V is in R^(d_model x d_v), and W^O is in R^(h*d_v x d_model). Multi-head attention lets the model jointly attend to information from different representation subspaces at different positions; a single attention head's averaging would inhibit this.

In the paper, h = 8 parallel attention heads (layers) are used, with d_k = d_v = d_model / h = 64. Because each head's dimension is reduced, the total computational cost is similar to single-head attention using the full dimensionality.

#### 6.3.3 The three ways multi-head attention is used in the Transformer

1. **Encoder-decoder attention layers**: queries come from the previous decoder layer; keys and values come from the encoder's output. This lets every decoder position attend over all input-sequence positions, mirroring typical encoder-decoder attention in earlier sequence-to-sequence models.
2. **Encoder self-attention layers**: queries, keys, and values all come from the same place, the output of the previous encoder layer. Every encoder position can attend to all positions in the previous encoder layer.
3. **Decoder self-attention layers**: each decoder position can attend to all decoder positions up to and including itself. Leftward information flow (attending to future positions) must be prevented to preserve the auto-regressive property; this is implemented inside scaled dot-product attention by masking out (setting to negative infinity) all softmax inputs that correspond to illegal (future) connections.

### 6.4 Position-wise Feed-Forward Networks

Each encoder and decoder layer also contains a fully connected feed-forward network applied identically and separately to each position:

**FFN(x) = max(0, x W1 + b1) W2 + b2**

This is two linear transformations with a ReLU activation in between; the linear transformations are the same across positions within one layer but differ from layer to layer (equivalent to describing this as two convolutions with kernel size 1). Input/output dimensionality is d_model = 512; the inner layer has dimensionality d_ff = 2048.

### 6.5 Embeddings and Softmax

Learned embeddings convert input tokens and output tokens into vectors of dimension d_model. A learned linear transformation plus softmax converts the decoder's output into predicted next-token probabilities. The same weight matrix is shared between the two embedding layers (input and output) and the pre-softmax linear transformation. In the embedding layers, these shared weights are multiplied by sqrt(d_model).

### 6.6 Positional Encoding

Because the Transformer has no recurrence and no convolution, it has no inherent notion of sequence order, so "positional encodings" are added to the input embeddings at the bottom of both the encoder and decoder stacks. The positional encoding has the same dimension d_model as the embeddings so the two can be summed. The paper uses sine and cosine functions of different frequencies:

**PE(pos, 2i) = sin( pos / 10000^(2i / d_model) )**

**PE(pos, 2i+1) = cos( pos / 10000^(2i / d_model) )**

where pos is the position and i is the dimension index. Each dimension of the positional encoding corresponds to a sinusoid, with wavelengths forming a geometric progression from 2*pi to 10000*2*pi. This function was chosen because it should let the model easily learn to attend by relative position, since for any fixed offset k, PE_(pos+k) can be expressed as a linear function of PE_pos. The paper also tried learned positional embeddings and found nearly identical results to the sinusoidal version, but kept the sinusoidal version because it may let the model extrapolate to sequence lengths longer than those seen during training.

### 6.7 Why self-attention (complexity comparison)

Comparing self-attention, recurrent, convolutional, and restricted self-attention layers on three criteria: total computational complexity per layer, the amount of computation that can be parallelized (minimum sequential operations), and the maximum path length between any two positions (shorter paths make long-range dependencies easier to learn):

| Layer Type | Complexity per Layer | Sequential Operations | Maximum Path Length |
|---|---|---|---|
| Self-Attention | O(n^2 * d) | O(1) | O(1) |
| Recurrent | O(n * d^2) | O(n) | O(n) |
| Convolutional | O(k * n * d^2) | O(1) | O(log_k(n)) |
| Self-Attention (restricted, neighborhood size r) | O(r * n * d) | O(1) | O(n/r) |

(n = sequence length, d = representation dimension, k = convolution kernel size, r = restricted self-attention neighborhood size.)

Self-attention connects all positions with a constant number of sequentially executed operations, while a recurrent layer needs O(n) sequential operations. Self-attention layers are faster than recurrent layers whenever the sequence length n is smaller than the representation dimension d, which is the usual case for the word-piece and byte-pair sentence representations used by state-of-the-art machine translation models. For very long sequences, the paper suggests restricting self-attention to a neighborhood of size r around each output position, which would increase maximum path length to O(n/r); this restricted variant was left as future work, not something implemented in this paper's experiments.

A single convolutional layer with kernel width k < n does not connect every pair of positions; doing so requires a stack of O(n/k) convolutional layers (contiguous kernels) or O(log_k(n)) layers (dilated convolutions), which lengthens the longest paths in the network. Convolutional layers are also generally more expensive than recurrent layers by a factor of k, though separable convolutions reduce this. As a side benefit, the paper notes that self-attention can yield more interpretable models: individual attention heads appear to learn distinct behaviors, some related to the syntactic and semantic structure of sentences (illustrated in the paper's attention visualizations, e.g., one encoder self-attention layer following the long-distance dependency completing the phrase "making ... more difficult," and other heads apparently performing anaphora resolution on the pronoun "its").

### 6.8 Training setup

- Trained on WMT 2014 English-German (about 4.5 million sentence pairs, byte-pair encoded into a shared ~37,000-token vocabulary) and the larger WMT 2014 English-French dataset (36 million sentences, split into a 32,000 word-piece vocabulary).
- Sentence pairs were batched by approximate sequence length; each training batch held about 25,000 source tokens and 25,000 target tokens.
- Hardware: one machine with 8 NVIDIA P100 GPUs. Base models: ~0.4 seconds per training step, 100,000 steps total (about 12 hours). Big models: ~1.0 second per step, 300,000 steps (about 3.5 days).
- Optimizer: Adam, with beta1 = 0.9, beta2 = 0.98, epsilon = 1e-9. Learning rate schedule:

  **lrate = d_model^(-0.5) * min( step_num^(-0.5), step_num * warmup_steps^(-1.5) )**

  This increases the learning rate linearly for the first warmup_steps (set to 4000) training steps, then decreases it proportionally to the inverse square root of the step number.
- Regularization: (1) residual dropout applied to each sub-layer's output before it is added back to the sub-layer input and normalized, and to the sums of embeddings and positional encodings at the bottom of both stacks (base model dropout rate P_drop = 0.1); (2) label smoothing with epsilon_ls = 0.1, which hurts perplexity (the model learns to be less certain) but improves accuracy and BLEU score.

### 6.9 Key results

- **Machine translation, WMT 2014 English-to-German**: the big Transformer model reaches a new state-of-the-art BLEU score of **28.4**, beating the best previously reported models (including ensembles) by more than 2.0 BLEU, while training in 3.5 days on 8 P100 GPUs. Even the base Transformer model surpasses all previously published single models and ensembles, at a fraction of their training cost.
- **Machine translation, WMT 2014 English-to-French**: the big Transformer reaches a BLEU score of **41.8** (also cited as 41.0 in the results discussion; 41.8 is the headline number in the comparison table), a new single-model state of the art, trained at less than 1/4 the training cost of the previous best single model. This run used dropout P_drop = 0.1 instead of 0.3.
- Base models were built by averaging the last 5 checkpoints (written at 10-minute intervals); big models averaged the last 20 checkpoints. Inference used beam search with beam size 4 and length penalty alpha = 0.6; maximum output length was capped at input length + 50, with early termination when possible.
- **Model variation experiments** (Table 3 of the paper) on the English-to-German development set: reducing to a single attention head costs about 0.9 BLEU versus the best configuration, and quality also drops with too many heads; reducing the attention key size d_k hurts quality (suggesting a more sophisticated compatibility function than plain dot product could help); bigger models perform better; dropout is very helpful against overfitting; and replacing the sinusoidal positional encoding with learned positional embeddings gives nearly identical results to the base model.
- **English constituency parsing** (a task the Transformer was not specifically designed for, used to test generalization): a 4-layer Transformer with d_model = 1024, trained on the Wall Street Journal (WSJ) portion of the Penn Treebank (about 40,000 training sentences), reached an F1 of 91.3 in the WSJ-only discriminative setting, and 92.7 in a semi-supervised setting (using an additional ~17 million sentences), outperforming all previously reported models in that table except the Recurrent Neural Network Grammar (93.3, generative) and a multi-task model (93.0). Notably, the Transformer outperformed the BerkeleyParser even when trained only on the 40K-sentence WSJ set, in contrast to prior RNN sequence-to-sequence models which had not reached state-of-the-art results in that small-data regime.

### 6.10 Conclusion of the paper

The Transformer is presented as the first sequence transduction model based entirely on attention, replacing the recurrent layers typically used in encoder-decoder architectures with multi-headed self-attention. It trains significantly faster than recurrent- or convolution-based architectures and achieved new states of the art on both WMT 2014 English-to-German and English-to-French translation. The authors state plans to extend the Transformer to non-text input/output modalities (images, audio, video), to investigate local/restricted attention mechanisms for large inputs and outputs, and to explore making generation less sequential.

---

## 7. Using BERT: BERTopic and Lexicon-Based Sentiment Analysis (Group 5 paper)

Paper: "Extracting Topics and Sentiments from Baguio City Social Media Using BERTopic and Vader Lexicon-based Sentiment Analysis," by Arevalo, Bayquen, Cayton, De los Trinos, and Fernandez, Computer Science Department, SAMCIS, Saint Louis University, Baguio City. This is a student research paper (also the subject of the Paper Review and Presentation Exercise assigned to Group 5) that combines a transformer-based topic modeling technique (BERTopic) with a lexicon-based sentiment scoring technique (VADER), customized for Tagalog and Ilocano.

### 7.1 Motivation and gap

Baguio City is a major Philippine tourist destination and an education hub for the Cordillera region, with heavy social media activity (Facebook groups/pages, Twitter, Instagram, YouTube, Reddit) around travel, renting, tertiary education, and tourism. Two gaps the paper targets:
1. No customized lexicon-based sentiment model that handles, for Tagalog and Ilocano, the same five heuristics VADER handles for English: capitalization, punctuation, degree modifiers, polarity shift due to "but," and negation.
2. No social media tool giving moderators visibility into behavior/sentiment trends and patterns.

The paper frames itself against prior work (citing a study by Manuel Garcia on COVID-19 tweet sentiment in Metro Manila) that translated Tagalog/Taglish text into English before analysis, which the authors argue loses context and sentiment accuracy. Their approach instead scores Tagalog and Ilocano terms directly and extends coverage beyond residents to tourists as well.

### 7.2 What BERT and BERTopic are (as explained in the paper)

**BERT** (Bidirectional Encoder Representations from Transformers) is a large language model introduced in 2018. It is bidirectional: it understands a word using the context of the words that come both before and after it, which lets it capture contextual information and semantic relationships (e.g., disambiguating "bank" in "river bank" versus "bank loan").

**BERTopic** is the topic-modeling technique built on top of BERT. It leverages BERT embeddings together with a class-based TF-IDF approach to form dense clusters, which become interpretable topics; the technique retains important words in each topic's description.

**Maximal Marginal Relevance (MMR)**: an auxiliary tool inside the BERTopic library used to diversify the keywords chosen to represent a topic. Left alone, BERTopic can treat near-duplicate forms (like a singular and its plural) as unrelated words filling up a topic's keyword list. MMR scores each candidate keyword on both its similarity to the document and its dissimilarity to keywords already chosen, so the final keyword set for a topic is more diverse. The paper set the MMR diversity parameter to 0.8, judged to give the most diverse keyword set while keeping cohesive meaning within each topic.

### 7.3 Data and preprocessing

- **Topic modeling dataset**: comments and replies (not original posts, since replies tend to carry more subjective opinion) mentioning Baguio City, scraped from Facebook, Twitter, Instagram, YouTube, and Reddit.
- **Word-lexicon dataset**: two Kaggle sources combined. The first is the "sentiments" dataset from R's Tidytext package, listing Tagalog and Ilocano words each marked positive, negative, or left blank for neutral. The second is a broader multilingual sentiment dataset, from which only the Tagalog subset was extracted and split into positive_tl.txt and negative_tl.txt.
- **Topic-modeling preprocessing**: tokenization, stop word removal, stemming, lowercasing, and removal of rare/overly common words.
- **Sentiment preprocessing**: annotated words were assigned a polarity of -1 (negative) or 1 (positive) so VADER (which works on numeric scores) could use them. For words in the raw data that lacked a local-language sentiment annotation, the researchers used the accompanying English-translation column, looked that English word up in VADER's existing lexicon, and copied its score onto the local-language word.

### 7.4 BERTopic pipeline specifics

- Left unconstrained, BERTopic generated roughly 200 to 300 raw topics.
- `reduce_topics` brought this down to 100 topics.
- The team manually read and labeled all 100 topics against **15 researcher-defined main categories**: Outlier, Parks and Recreation, Public Transportation/Traffic Management, Waste Management, Housing, Public Safety, Infrastructure (Roads, City Services, etc.), Healthcare, Tourism/Travel Destinations, Food, Environment, Arts and Culture, Community Development, Taxes/Budget and Finance, and City Ordinances. "Outlier" catches subtopics that don't fit a specific niche.
- Merge rule: a subtopic joins a main category only if it was labeled as belonging to that category by the annotators more than twice.

### 7.5 VADER heuristics, extended with local-language equivalents

VADER (Valence Aware Dictionary and sEntiment Reasoner) uses five heuristics (Hutto & Gilbert, 2014):

1. **Capitalization**: fully capitalized words get more weight. "I AM VERY HAPPY" scores more positive than "I am very happy."
2. **Punctuation**: punctuation like "!!!" intensifies sentiment. "I like it!!!" scores stronger than "I like it."
3. **Degree modifiers**: words that intensify or weaken the following word's sentiment. Tagalog examples: "sobra" intensifies ("Sobrang saya ko ngayon"), "medyo" weakens ("Medyo masaya ako ngayon"), "malaki" amplifies (as in "malaking abala sa mga tao," amplifying "abala"). The team gathered 123 such adverbs (Tagalog: "pang-abay na pamamaraan").
4. **Polarity shift due to "but"**: a sentence's dominant sentiment often shifts to whatever follows "but." Tagalog example: "Mahal kita, pero ayaw na kitang makasama" starts positive ("Mahal kita") but the clause after "pero" is the dominant (negative) sentiment. Shift words incorporated: pero, ngunit, subalit, bagkus (Tagalog) and ngem (Ilocano).
5. **Negation**: words like "not," "never," "no" flip sentiment. Tagalog example: "Hindi ka matinong kausap" would be misread as mildly positive if "hindi" (not) were ignored, since "matino" usually connotes something positive; correctly handling the negation flips the sentence to negative. The team gathered 62 negation words/phrases, with roughly 70% being variants of "hindi."

### 7.6 Evaluation methodology

500 comments were sampled and independently scored by every team member (positive/negative/neutral category, plus an intensity score from 0 to 100); the per-member scores were averaged into a consensus ground-truth score per comment. Two evaluation modes were run: **binary** (positive vs. negative) and **multiclass** (adding neutral), each measured with accuracy, precision, recall, F1, and both Mean Absolute Error (MAE) and Root Mean Square Error (RMSE) against the consensus scores.

### 7.7 Results

**Topic distribution** (Table 1, percentage of subtopics falling under each main category; these can sum past 100% because subtopics can overlap between categories, e.g., Infrastructure and Public Transportation): Outlier 0.52, Tourism/Travel Destinations 0.16, Arts and Culture 0.10, Public Safety 0.08, Infrastructure 0.08, Parks and Recreation 0.08, Public Transportation/Traffic Management 0.06, Food 0.04, Housing 0.03, Community Development 0.02, Environment 0.01, Healthcare 0.01. Waste Management and Taxes/Budget had no matching subtopics at all in this run; City Ordinances is not listed in the results table.

**Topic coherence** (a measure of how semantically similar the words within a topic are to each other; higher is more interpretable): no topic-count limit -> 0.3552; reduced to 100 topics -> 0.3547; reduced to 15 topics -> 0.3342; reduced to 9 predefined topics -> 0.4043 (described in the paper as "moderate" coherence).

**Intertopic distance map**: showed two regions, an upper-left cluster of several small, highly similar topics, and a lower-right region containing one very large topic overlapped by another, interpreted as two topics that are distantly related but contain some unique content each.

**Sentiment analysis, multiclass (with neutral)**: Accuracy 0.5598, Precision 0.6698, Recall 0.5598, F1 0.5822. MAE: positive 45.25, negative 6.94, neutral 43.83. RMSE: positive 63.82, negative 20.61, neutral 58.53.

**Sentiment analysis, binary (positive/negative only)**: Accuracy 0.7274, Precision 0.8700, Recall 0.7274, F1 0.7734 (MAE/RMSE figures reported for binary matched the positive/negative figures from the multiclass run).

The paper attributes the multiclass performance drop mainly to manual labeling behavior: because annotators labeled by hand, they tended to mark mildly positive comments as neutral, inflating the neutral class and hurting multiclass performance and interpretability.

### 7.8 Conclusions and the paper's own recommendations

The paper concludes that coherence "improved slightly" through manual topic merging into the 15 predefined categories (though it also notes overlapping main topics may have hurt coherence); that binary sentiment classification performed properly on test data; that the model struggled specifically with neutrality; and that error metrics showed particular trouble predicting how strongly positive a positive comment actually was. The authors' own recommendations: (1) explore other BERTopic clustering tools such as CountVectorizer, TF-IDF, and hierarchical clustering to improve topic granularity; (2) build a dedicated Tagalog/Ilocano sentiment lexicon scored by native speakers, rather than mapping unannotated local words to existing English VADER entries.

### 7.9 Critical counterpoints raised in the class review (useful exam material on evaluating NLP papers)

These were developed by the reviewing group (not by the original authors) and are worth knowing as examples of methodological critique of an NLP paper:

- The stated conclusion ("manual merging improved coherence") is contradicted by the reported numbers: 15 merged topics scored 0.3342 coherence, which is *lower* than both the unconstrained run (0.3552) and the 100-topic run (0.3547). Only the 9-predefined-topic configuration (0.4043) beat the baselines, and that configuration is not the one the study is built around.
- The coherence metric variant used (e.g., c_v, u_mass, c_npmi) is never named, and BERTopic's underlying UMAP/HDBSCAN clustering settings are not reported, which blocks reproducibility.
- 52% of the corpus fell into the "Outlier" bucket (no assigned topic), which is treated as incidental rather than as a tuning target; BERTopic offers `reduce_outliers` and adjustable HDBSCAN minimum cluster size specifically to address this.
- The multiclass sentiment collapse is blamed on annotators rather than on the labeling protocol; since a 0-100 intensity score was already being collected per comment, a fixed numeric band (e.g., 40-60) could define "neutral" objectively instead of relying on each annotator's separate judgment call, and no inter-annotator agreement statistic (such as kappa) is reported.
- Words scored by mapping to English VADER entries (via the English-translation column) reintroduces the exact translation-based loss of nuance the paper's introduction argues against, and the fraction of the lexicon scored this way is never disclosed.
- There is no baseline comparison against plain English VADER run on machine-translated comments, so the central claim (native-language scoring beats translation) is asserted but never directly tested.
- The rule for merging 100 topics into 15 categories ("labeled ... more than twice by the annotators") does not specify how many annotators there were or how ties are broken, and the actual 100-to-15 topic mapping is not published.
- Table 1 reports only 12 of the 15 defined categories; Waste Management and Taxes/Budget are explained as having no subtopics, but City Ordinances is simply absent with no explanation.
- The binary and multiclass evaluations report identical MAE/RMSE figures for positive and negative classes, which is suspicious since removing the neutral class should change how the remaining errors are computed.
- The underlying social media corpus itself (size, date range, per-platform breakdown, scraping consent) is never described, so the representativeness of the 500-comment test sample relative to the full dataset cannot be assessed.

---

## 8. NLP for Governance / Sentiment Analysis: InfoSentiA

Paper: "InfoSentiA: A Local Legislation Information Dissemination System with Sentiment Analysis Driven Feedback Processing," by Espiritu, Eslao, Donglawen, Fama, Garcia, Genove, M.A. Zheng, and M.J. Zheng, Saint Louis University. This is an applied e-governance system built around NLP-driven sentiment analysis of citizen feedback, and it is one of the syllabus's named case studies for "the Application of NLP in electronic governance."

### 8.1 Problem being solved

Ordinances and resolutions are part of the law governing local government units (LGUs). Citizen awareness of local legislation improves enforcement, and citizen feedback helps the LGU's legislative body decide on amendments, repeals, or follow-up legislation. The specific office studied, the Research Division of the Sangguniang Panlungsod (RDSP, the legislative branch of Baguio City government, mandated by Republic Act 7160/the Local Government Code of 1991), faced three operational problems:
1. **Time-intensive manual processes**: personnel had to travel to respondents and hand out questionnaires in person.
2. **Insufficiency of data collected**: low questionnaire return rates hurt data quality.
3. **Weak information dissemination**: many citizens were unaware of current local legislation.

### 8.2 What the system does

InfoSentiA is a web application with two main modules:
- **Public module**: lets the public view disseminated ordinances/resolutions, answer monitored questionnaires, view collated results, search ordinances/resolutions/questionnaires by keyword, and view static/dynamic pages.
- **Admin module** (used by RDSP personnel): manages disseminated ordinances/resolutions (including watermarked uploaded documents), manages questionnaires, generates downloadable statistical/status reports (Excel/XLS), views public comments/suggestions, and runs sentiment analysis over comments, suggestions, and survey responses.

The system integrates with Facebook via the Graph API, so a newly posted ordinance is automatically pushed to a Facebook page, and comments on that post are collected back into the system for analysis.

### 8.3 The NLP/sentiment component: Project Lengua

Sentiment analysis is handled by a separate subsystem called **Project Lengua**, exposed as a RESTful API (built on Node.js, Express, and MongoDB, versus the main system's PHP/Laravel/MySQL stack). Project Lengua does sentiment scoring using an **AFINN-based** lexicon approach (a supervised text classification / lexicon-scoring method): each word in its maintained word-list is stored with a word token, a dialect label (used for sorting/filtering), and a numeric AFINN-style score ranging from -5 to 5 (Nielsen, 2011, is cited as the AFINN source). Given input sentences, the API returns, per sentence, a score, a comparative value, and the list of tokens/words along with their contribution to the sentiment, letting the caller derive an overall polarity (positive, negative, or neutral). The API supports both a GET request for a single sentence and a POST request for multiple sentences, and is designed to be reusable by any external application, not just the main InfoSentiA system.

### 8.4 System workflow (end to end)

1. RDSP staff post a new ordinance or resolution into the admin module.
2. The system immediately posts that entry to both the public module and a Facebook page.
3. Comments and suggestions accumulate on the Facebook page and in the public module.
4. The main system sends all collected comments/suggestions to the Project Lengua API for scoring.
5. Project Lengua returns sentiment results (score, comparative, polarity) for each comment.
6. The admin module displays the aggregated results as a donut chart (example in the paper: "Ordinance Pulse," showing counts of positive, negative, and neutral sentiments for a given legislation).

### 8.5 Tools and technologies used

Frontend: HTML5, CSS3, JavaScript, Bootstrap, jQuery, Vue.js. Backend: PHP, Laravel, Node, Express. Mailing: SendGrid, Mailtrap. Web server: WampServer. Hosting: Heroku. Databases: MySQL Workbench/phpMyAdmin (relational) and MongoDB (NoSQL, used by the Project Lengua subsystem). Sentiment analysis: the Project Lengua RESTful API (AFINN-based). Social media integration: Facebook Graph API. File storage: Google Drive. Project collaboration: Trello, Git, GitHub. Both the main system and Project Lengua are deployed using a client-server architecture implemented with the Model-View-Controller (MVC) pattern, chosen to support code refactoring and maintainability.

### 8.6 Development approach

The team used an **evolutionary development model**, chosen because it does not require fully clear requirements up front; requirements were refined over repeated feedback cycles with the client (the Research Division), gathered through interviews, direct observation of city hall legislative sessions, and iterative demonstration of a semi-functional prototype. Non-functional requirements emphasized security (encrypted credentials, role-based login), availability (accessible on any web-connected device), usability, and legal compliance (no disclosure of personal information, in line with the Data Protection Act).

### 8.7 Conclusions and recommendations

The paper frames citizens as the main catalyst for a government fulfilling its legislative responsibilities, and argues an e-government system integrated with popular social media (here, Facebook) both widens legislation dissemination and eases feedback collection. NLP components used are explicitly named as tokenization, lexicon building/utilization, and sentiment analysis, which together let citizen feedback be classified into general public sentiment categories to inform legislative decision-making. At the time of writing, a functioning system was already deployed and under testing. Recommendations for future work: (1) integrate additional social media channels beyond Facebook (e.g., Twitter, LinkedIn); (2) add a feature letting citizens report legislation violations directly.

---

## 9. Papers Assigned for Group Reading and Presentation

For context on the broader NLP literature landscape referenced across the course (only Groups 4, 5, and 8's papers were read in full for this document; the rest are listed here for topic awareness, since the exercise only required reading the paper assigned to one's own group):

- **Group 1**: "Language Mapping of the Cordillera Administrative Region Using Relational Model" (International Journal of Computing Sciences Research).
- **Group 2**: "Cloud-Based RoBERTa NLP for Assessing Academic Institutions' Contributions to Sustainable Development Goals" (IEEE Symposium on Computers & Informatics, ISCI). Note: RoBERTa is also referenced in the syllabus's midterm-period topic list, alongside BERT and Rasa, as an emerging NLP platform to study.
- **Group 3**: "Bi-directional Ilocano-English Language Translator Using Customized Moses Statistical Machine Translation System (SMTS)."
- **Group 4**: "InfoSentiA: A Local Legislation Information Dissemination System with Sentiment Analysis Driven Feedback Processing" (fully summarized in Section 8 above).
- **Group 5**: "Extracting Topics and Sentiments from Baguio City Social Media Using BERTopic and Vader Lexicon-based Sentiment Analysis" (fully summarized in Section 7 above).
- **Group 6**: "LexiLoko 2.0: A Lexicon-Integrated Neural Machine Translation Framework for Advancing Iloko Language Preservation."
- **Group 7**: "Comparative Evaluation of Tagalog Part-of-Speech Taggers."
- **Group 8**: "Modeling Kankanaey Adjective Inflections" (fully summarized in Section 4.4 above; also referenced independently in the course's morphology lecture materials).

The paper-review activity's stated objectives (applicable across all groups) were: (1) review and specify the relevance, if any, of an NLP paper written or published by Saint Louis University-affiliated authors, and (2) deliver a short oral presentation (10 to 15 minutes) on the paper's content, emphasizing NLP-related concepts and theories, and integrating the reviewing group's own feedback and counterproposals.
