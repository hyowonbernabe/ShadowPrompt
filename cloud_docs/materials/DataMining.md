# CSE 15 Data Mining, Midterm Exam Reviewer

Exam scope from the instructor: text analytics, topic modeling, sequential pattern mining (SPM, application and concepts), and the lab activities. Written exam, closed formulas, calculator allowed, solutions on paper.

Sources: the three midterm slide decks (Text Analytics, Topic Modeling (LDA), Sequential Pattern Mining 26-27), the Midterm Lec Activity (sentiment steps), Midterm Activity 2 (time series forecasting), and the Midterm Summative Activity (topic modeling, proximity, rule-based sentiment). Prelim topics show up only where a midterm topic leans on them.

Contents:

1. Text Analytics (concepts, preprocessing, bag of words, TF, IDF, TF-IDF)
2. Text Classification and Sentiment Analysis (Naive Bayes, SVM, Random Forest, lexicon methods, VADER, TextBlob, SentiWordNet)
3. Topic Modeling and LDA
4. Sequential Pattern Mining (GSP fully worked, SPADE, PrefixSpan, sequential rules)
5. Proximity measures used in the midterm (cosine, Jaccard, Euclidean)
6. Lab Activities
7. Formula Sheet (everything in one place)
8. Likely Exam Questions and Quick Self-Check

---

# 1. Text Analytics

## 1.1 What text analysis is

- **Text analysis**: the automated process of turning large volumes of unstructured text into quantitative data to uncover insights, trends and patterns.
- It deals with **unstructured data**: conversations, declarations, tweets, comments. This kind of data does not fit the row and column structure of relational databases, and it is the vast majority of data in the real world.

## 1.2 Text formats

| Format | What it looks like | Slide example |
|---|---|---|
| **Structured** | Fields in a table (NAME, BANK, BRANCH, BRAND, LOAN AMOUNT, TERMS) | John Cruz, ABC Bank, Abanao, Toyota, 1.2M, 15 years |
| **Free text** | Plain sentences | "Mr. John Cruz purchased Toyota Sedan. He purchased the car thru a bank loan from ABC Bank. The loan amount at 1.2M is payable in 15 years." |
| **Multimedia / unstructured** | Text mixed with markup, links, media | `<a href>John Cruz</a> purchased <a href>a car</a> from <a href>Hyundai</a>` |

## 1.3 Drawbacks: language is tricky

- **Context**: the same words mean different things in different situations.
- **Syntax and semantics**: structure (syntax) and meaning (semantics) both matter.
- **Spelling errors**.

## 1.4 Difficulties of NLP (natural language processing, getting computers to work with human language)

| Difficulty | Meaning | Slide example |
|---|---|---|
| **Word-level ambiguity** | One word, several parts of speech or meanings | "design" can be a noun or a verb (ambiguous POS). "root" has multiple meanings (ambiguous sense). |
| **Syntactic ambiguity** | The sentence structure can be read more than one way | "natural language processing". "A man saw a boy with a telescope." (who has the telescope?) |
| **Anaphora resolution** | Working out what a pronoun refers to | "John persuaded Bill to buy a TV for himself." (himself = John or Bill?) |
| **Presupposition** | The sentence assumes something not stated | "He has quit smoking." implies he smoked before. |

## 1.5 NLP process

**Data pre-processing steps (from the slides, in order):**

1. Convert accented characters
2. Expand contractions
3. Tokenization
4. Stop words removal
5. Stemming
6. Lemmatization
7. Parts of speech tagging

**Modelling techniques (list on the slide):** Bag of Words, TF-IDF, Word Embeddings, Sentiment Analysis, Keywords Extraction, Named Entity Recognition, Topic Modelling, Summarization.

## 1.6 Pre-processing steps in detail

**Convert accented characters**: replace accented letters with plain ones. Latté -> latte, Café -> cafe.

**Expand contractions**: don't -> do not, can't -> can not.

**Tokenization**: splitting a phrase, sentence, paragraph or whole document into smaller units (words or terms). A **token** is one of those units, usually a word.
- Helps interpret the meaning of the text by analyzing the words in it.
- Lets you count the number of words in a text.
- Example: "I saw a cat" -> tokens "I", "saw", "a", "cat".

**Stop words removal**: stop words are common words that add little to the meaning of a sentence (the, is, in, for, to). They act as a bridge and keep sentences grammatical, but they are filtered out before processing.
- Example: "The quick brown fox jumped over the lazy dog." -> "quick brown fox jumped lazy dog".

**Stemming**: cutting a word down to its root (stem) by removing prefixes and suffixes. It is a crude rule, so the stem may not be a real word.
- Stemmers named in the slides: **Porter stemmer**, **Krovetz stemmer**.
- Why stem: fewer input dimensions, ML techniques work better, training data becomes denser, the dictionary gets smaller, and words are normalized across the document.

| Word | Suffix removed | Stem |
|---|---|---|
| cats | s | cat |
| caring | ing | car (wrong meaning, a classic stemming mistake) |
| programming | ing | programm (not a real word) |

**Lemmatization**: also reduces inflected words to their root, but it returns the **lemma**, the real dictionary form, based on the word's intended meaning. It does a **morphological analysis** (looks at how the word was formed: tense, number, part of speech).
- Tools named: WordNet Lemmatizer with NLTK, spaCy Lemmatizer, TextBlob Lemmatizer, Pattern Lemmatizer, Stanford CoreNLP Lemmatizer.
- Slide table: helps (third person singular) -> help, helping (present participle) -> help, flies (noun/verb) -> fly, better (adjective/adverb) -> good.

**Stemming vs lemmatization**: stemming chops letters by rule and can give non-words (programm, car from caring). Lemmatization looks up the dictionary form using meaning and part of speech, so it is slower but correct.

**Parts of Speech (POS) tagging**: assigning a part of speech to each word in a sentence. Done at the token level.
- "The quick brown fox jumps over the lazy dog": The = determiner (DT), quick = adjective (JJ), brown = adjective (JJ), fox = noun (NN), jumps = verb, 3rd person singular present (VBZ), over = preposition (IN), the = DT, lazy = JJ, dog = NN.

## 1.7 Text classification approaches

**Text classification**: the process of putting text into categories. Three approaches:

1. **Rule-based system**: texts are grouped using handcrafted linguistic rules. Users define a list of words for each group. Example: Robredo, Lacson, Marcos -> politics. Christianity, Islam, Atheism -> religion.
2. **Machine-based system**: the classification is learned from pre-labeled datasets. Uses bag of words as features. Flow: text -> feature extractor -> features -> ML algorithm (with tags) -> classifier model.
3. **Hybrid system**: a mix of rule-based and machine-based. Uses the list of words to label the dataset, and the classification is improved over several rounds by updating the word list.

## 1.8 Bag of words (BoW)

- **Bag of words**: a vector that holds the frequency of each word from a predefined dictionary (word list).
- Keeps a count of total occurrences of the most frequently used words.
- One of the methods used to turn tokens into a set of features.
- **Ignores grammar and word order.**
- Pipeline: Clean Text -> Tokenize -> Build Vocab -> Generate Vectors.
- **Vocabulary**: the list of words in a corpus (a **corpus** is the whole collection of documents).
- **Feature vector**: an n-dimensional vector of numbers that represents some object.

Slide example (stopwords removed, so 6 vocabulary words):

- Sentence 1: "This is a good job. I will not miss it for anything"
- Sentence 2: "This is not good at all"

| | good | job | miss | anything | not | all |
|---|---|---|---|---|---|---|
| Sentence 1 | 1 | 1 | 1 | 1 | 1 | 0 |
| Sentence 2 | 1 | 0 | 0 | 0 | 1 | 1 |

(The slide table has a couple of cells off. Count it yourself: sentence 2 has good, not, all.)

## 1.9 N-grams

- **N-gram**: a sequence of N tokens.
- **2-gram (bigram)**: two-word sequence, like "really good", "not good", "your homework".
- **3-gram (trigram)**: three-word sequence, like "not at all", "turn off light".
- Bigrams of "This is not good at all": "This is", "is not", "not good", "good at", "at all".
- Count rule: a sentence with T tokens has `T - n + 1` n-grams. "This is not good at all" has 6 tokens, so 6 - 2 + 1 = 5 bigrams.
- Why use them: unigrams lose word order, so "not good" looks like "good". Bigrams keep that local order.

## 1.10 Term weights

Three slide documents used for every term-weight example:

1. The quick brown fox jumped over the lazy dog.
2. The brown dog and brown fox are lazy.
3. The fox is brown.

**Binary term occurrence**: 1 if the term is in the document, 0 if not. Ignores how many times.

| | quick | brown | fox | jump | over | lazy | dog |
|---|---|---|---|---|---|---|---|
| 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| 2 | 0 | 1 | 1 | 0 | 0 | 1 | 1 |
| 3 | 0 | 1 | 1 | 0 | 0 | 0 | 0 |

**Term frequency (TF) as raw count**: number of times the term occurs in the document.

| | quick | brown | fox | jump | over | lazy | dog |
|---|---|---|---|---|---|---|---|
| 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| 2 | 0 | **2** | 1 | 0 | 0 | 1 | 1 |
| 3 | 0 | 1 | 1 | 0 | 0 | 0 | 0 |

## 1.11 TF-IDF

**TF-IDF (Term Frequency, Inverse Document Frequency)**: a score for how important a word is to one document within the whole corpus.
- Used to score terms for information retrieval (IR) or summarization.
- Reflects how relevant a term is in a given document.
- Takes into account how many times the word appears in the document, offset by how many documents in the corpus contain it.
- **Emphasizes terms that are frequent in a document but rare across the collection.**

**Formulas (memorize):**

```
TF(t, d)  = (number of times term t appears in document d) / (total number of terms in document d)
            also written TF(i,j) = n(i,j) / sum over k of n(k,j)

IDF(t, D) = log( N / df(t) )
            N     = total number of documents in the corpus D
            df(t) = number of documents that contain term t
            log   = base 10 in the slides

TF-IDF(t, d) = TF(t, d) x IDF(t, D)
```

- TF: a higher frequency suggests greater importance within that one document.
- IDF: lowers the weight of words common across many documents and raises the weight of rare words. A term in fewer documents is more specific.
- A term in **every** document gets IDF = log(N/N) = log 1 = 0, so its TF-IDF is 0 no matter how often it appears.

### Worked example (exactly as the slides do it)

**Step 1, preprocessing**: lowercase, remove stop words (the, and, are, is, over).

**Step 2, vocabulary**:

| Doc | Terms after preprocessing | Total terms |
|---|---|---|
| D1 | quick, brown, fox, jumped, lazy, dog | 6 |
| D2 | brown, dog, brown, fox, lazy | 5 (brown counts twice) |
| D3 | fox, brown | 2 |

N = 3.

**Step 3, TF**:

| Term | D1 | D2 | D3 |
|---|---|---|---|
| quick | 1/6 | 0 | 0 |
| brown | 1/6 | 2/5 | 1/2 |
| fox | 1/6 | 1/5 | 1/2 |
| jumped | 1/6 | 0 | 0 |
| lazy | 1/6 | 1/5 | 0 |
| dog | 1/6 | 1/5 | 0 |

**Step 4, IDF** (log base 10):

| Term | df | IDF |
|---|---|---|
| quick | 1 | log(3/1) = 0.477 |
| brown | 3 | log(3/3) = 0 |
| fox | 3 | log(3/3) = 0 |
| jumped | 1 | log(3/1) = 0.477 |
| lazy | 2 | log(3/2) = 0.176 |
| dog | 2 | log(3/2) = 0.176 |

**Steps 5 and 6, TF x IDF**:

- D1: quick = 1/6 x 0.477 = 0.0795. jumped = 0.0795. lazy = 1/6 x 0.176 = 0.0293. dog = 0.0293. brown = fox = 0.
- D2: lazy = 1/5 x 0.176 = 0.0352. dog = 0.0352. Everything else 0 (brown has TF 2/5 but IDF 0).
- D3: all 0 (it only has brown and fox).

**Final TF-IDF matrix**:

| | quick | brown | fox | jumped | lazy | dog |
|---|---|---|---|---|---|---|
| D1 | 0.0795 | 0 | 0 | 0.0795 | 0.0293 | 0.0293 |
| D2 | 0 | 0 | 0 | 0 | 0.0352 | 0.0352 |
| D3 | 0 | 0 | 0 | 0 | 0 | 0 |

**Interpretation (slide wording, in short)**:
- quick and jumped are highest because they appear only in D1.
- brown and fox are 0 because they are in all documents, so they do not help tell documents apart.
- lazy and dog are moderate because they appear in 2 of 3 documents.

Calculator check: log(3) = 0.4771, log(1.5) = 0.1761.

### TF-IDF pitfalls

- Use log base 10 unless the question says otherwise. The slides use base 10.
- TF here is a **proportion** (count / document length), not the raw count. If the question shows raw counts, check what it asks.
- Count document length **after** stop word removal. D2 is 5, not 8.
- df counts **documents**, not occurrences. brown appears 4 times total but df(brown) = 3.
- High TF alone does not mean high TF-IDF (brown in D2).

### CountVectorizer vs TfidfVectorizer (sklearn)

- **CountVectorizer** gives the count of each word, by vocabulary index (plain bag of words).
- **TfidfVectorizer** weighs words by their importance over all documents.
- Slide example: train = ["The sky is blue.", "The sun is bright."] with English stop words. Count gives blue/sky = 1 in Doc1, bright/sun = 1 in Doc2. TF-IDF gives 0.707107 for each present word.
- Why 0.707 and not the hand formula: sklearn uses a smoothed IDF, `ln((1 + N) / (1 + df)) + 1`, then scales each row to length 1 (L2 normalization). Two equal weights in a row of length 1 give 1/sqrt(2) = 0.707. On paper, use the slide formula unless told otherwise.

---

# 2. Text Classification and Sentiment Analysis

## 2.1 Machine-based classifiers named in the slides

Naive Bayes, Support Vector Machine (SVM), Random Forest.

## 2.2 Naive Bayes

- Used for text classification and text analysis.
- Based on **Bayes' theorem**, which describes how conditional probabilities relate to each other.
- "Naive" because it assumes every word is independent of the others given the class.

```
Bayes' theorem:      P(L | features) = P(features | L) x P(L) / P(features)

Naive Bayes for text: P(c | d) proportional to P(c) x product over k = 1..n_d of P(t_k | c)

   P(c)        = prior probability of class c = (docs in class c) / (total docs)
   P(t_k | c)  = conditional probability of term t_k appearing in a document of class c
   n_d         = number of terms in document d

Laplace (add-one) smoothing, from the slide:
   P(w_k | c) = (n_k + 1) / (n + |vocabulary|)
   n_k          = times word w_k appears in all class-c documents
   n            = total number of words in all class-c documents
   |vocabulary| = number of distinct words in the whole training set
```

The +1 stops a single unseen word from making the whole product 0.

### Worked example (slide data)

| Doc | Text | Class |
|---|---|---|
| 1 | I loved the movie | + |
| 2 | I hated the movie | - |
| 3 | A great movie, good movie | + |
| 4 | Poor acting | - |
| 5 | Great acting, a good movie | + |

Classify the new document **"I hated the poor acting"** (a common test sentence for this table; the slide itself does not give one).

1. Vocabulary (lowercase, keep all words): i, loved, the, movie, hated, a, great, good, poor, acting. |V| = 10.
2. Priors: P(+) = 3/5 = 0.6. P(-) = 2/5 = 0.4.
3. Positive word counts: i 1, loved 1, the 1, movie 4, a 2, great 2, good 2, acting 1. n(+) = 14.
4. Negative word counts: i 1, hated 1, the 1, movie 1, poor 1, acting 1. n(-) = 6.
5. Smoothed probabilities:
   - Positive denominator 14 + 10 = 24: P(i|+) = 2/24, P(hated|+) = 1/24, P(the|+) = 2/24, P(poor|+) = 1/24, P(acting|+) = 2/24.
   - Negative denominator 6 + 10 = 16: P(i|-) = 2/16, P(hated|-) = 2/16, P(the|-) = 2/16, P(poor|-) = 2/16, P(acting|-) = 2/16.
6. Scores:
   - + : 0.6 x (2 x 1 x 2 x 1 x 2) / 24^5 = 0.6 x 8 / 7,962,624 = **6.03 x 10^-7**
   - - : 0.4 x (2 x 2 x 2 x 2 x 2) / 16^5 = 0.4 x 32 / 1,048,576 = **1.22 x 10^-5**
7. Negative is larger, so the document is **negative**.

Pitfalls: add |V| to the denominator, not 1. Use the class's total word count n, not the number of documents. Repeated words in the test document multiply in once per occurrence.

## 2.3 SVM (Support Vector Machine)

- Supervised learning algorithm for classification.
- Goal: create the best line or decision boundary that separates n-dimensional space into classes.
- **Hyperplane**: the boundary that separates the classes (the slide says "multiple line boundaries to separate classes"; the best one is the maximum margin hyperplane).
- **Support vectors**: the data points closest to the hyperplane.
- **Margin**: the distance between the support vectors and the hyperplane. SVM picks the hyperplane with the largest margin.

## 2.4 Random Forest

- Supervised learning algorithm for classification.
- Based on **ensemble learning**, combining many classifiers to solve a complex problem and improve the model.
- More trees in the forest gives higher accuracy and prevents overfitting.
- Assumptions:
  - The feature variables should hold actual values, so the classifier predicts accurately and does not guess.
  - The predictions of each tree must have very low correlation with each other.

## 2.5 Sentiment analysis (rule-based / lexicon-based approach)

- **Sentiment analysis**: classifying text by the emotion it seems to convey, usually positive, negative or neutral.
- Slide sentences to think about: "I like to travel" (positive), "I like to travel, sometimes" (weaker), "I do not dislike cabin cruise" (double negation), "It looks like it's going to rain" (like is not a sentiment here).
- **Applications**: market analysis, social media monitoring, customer feedback analysis, market research.
- **Lexicon**: the vocabulary of a group of people, a language or a field.
- Lexicon-based approach: words are labeled positive or negative (sometimes neutral) using a **valence dictionary** (a word list with sentiment scores). Example: "Good people sometimes have bad days." -> good = positive, bad = negative, the rest neutral.

**Two lexicon-based approaches:**

1. **Dictionary-based approach**: start with a few seed words, then expand the dictionary using an online dictionary, thesaurus or WordNet (adding synonyms and antonyms) until no new words can be added. Refine by manual inspection.
2. **Corpus-based approach**: finds the sentiment orientation of context-specific words.
   - **Statistical approach**: a word that shows up more often in positive text has positive polarity. More often in negative text, negative polarity. Equal frequency in both, neutral.
   - **Semantic approach**: gives sentiment values to words and to words that are semantically close to them (found through synonyms and antonyms).

**Lexicon-based tools in the slides:** TextBlob, VADER, SentiWordNet.

### TextBlob

`TextBlob(text).sentiment` gives two numbers:
- **Polarity**: -1 (negative) to +1 (positive). Says whether text is positive, negative or neutral.
- **Subjectivity**: 0 (fact) to 1 (opinion). Says whether the text is opinion-based or factual.

### VADER (Valence Aware Dictionary and sEntiment Reasoner)

- Built for social media text and informal language.
- Scores the polarity of each word by its emotional value, then combines the word scores into one score for the whole text.
- Uses the **compound score**, a normalized value between -1 and +1:

```
compound >  0.05            -> positive
compound < -0.05            -> negative
-0.05 <= compound <= 0.05   -> neutral

compound = S / sqrt(S^2 + alpha),  alpha = 15
   S = sum of the (adjusted) word valence scores
```

(The normalization formula is not on the slide, but I used it in the lec activity.)

### SentiWordNet

- A lexical resource that extends **WordNet** (a large English word database grouped by meaning).
- Adds sentiment scores to words. Each meaning of a word gets a positive, a negative and an objective score that add up to 1.

The step-by-step manual process for all three tools is in Section 6.1.

---

# 3. Topic Modeling and LDA

## 3.1 Topic modeling basics

- **Topic modeling**: an NLP technique for finding the topics in a document.
- In NLP, a **topic** is a set of words that "go together".
- It discovers patterns of words in a collection of documents.
- **Topic models** are statistical language models that uncover hidden structure in a collection of texts.
- Applications: text summarization, recommender systems, spam filters.
- Slide example: four news snippets mapped to topics Sports (Inside the NBA), Technology (satellites, Project Kuiper), Politics and Laws (Gensler, SEC, Ripple).

**Topic modeling algorithms named:**
- Latent Semantic Analysis (LSA, also called LSI)
- Probabilistic Latent Semantic Analysis (pLSA)
- **Latent Dirichlet Allocation (LDA)**

## 3.2 What LDA is

- An **unsupervised** clustering technique (no labels needed) commonly used for text analysis.
- Words are grouped into topics, and documents are described as collections of those topics.
- **Two key assumptions:**
  1. Documents are a mixture of topics.
  2. Topics are a mixture of tokens (words).
- Topics generate words using their probability distribution.
- Documents are described as a probability distribution of topics, and topics as a probability distribution of words.
- **Latent** means hidden. The topics are never observed, only the words are.

## 3.3 The Dirichlet distribution and alpha

- LDA places each document inside a **triangle (simplex)** whose corners are the topics (slide: Physics, Computer science, Biology). A document near a corner is mostly that one topic. A document in the middle is an even mix.
- Some documents have several topics. The distribution that describes these mixtures is the **Dirichlet distribution**, controlled by the parameter **alpha**.
- Alpha is usually a **k-dimensional vector**, one component per corner (topic). It controls the mix of topics in a document.

| alpha | Where samples fall | Meaning |
|---|---|---|
| alpha = 1 | Evenly spread over the triangle | Any mix equally likely |
| alpha > 1 | Gathered in the middle | Documents mix many topics |
| alpha < 1 | Pushed toward the corners | Each document is mostly one or a few topics |

The same idea applies to **beta** for topics over words: a second Dirichlet, with parameter beta, maps topics into word space.

## 3.4 LDA symbols (plate notation)

| Symbol | Meaning |
|---|---|
| **M** | Total number of documents in the corpus (outer box, e.g. M = 5) |
| **N** | Number of words in a document (inner box) |
| **K** | Number of topics (set in advance) |
| **w** | A word in a document (the only observed variable, shaded circle) |
| **z** | The latent (hidden) topic assigned to a word |
| **theta (θ)** | Topic distribution of a document (document-topic) |
| **phi (φ)** | Word distribution of a topic (topic-word) |
| **alpha (α)** | Dirichlet prior for the document-topic distribution |
| **beta (β)** | Dirichlet prior for the topic-word distribution |

Plate chain: alpha -> theta_m -> z_m,n -> w_m,n <- phi_k <- beta. The labels on the slide are document-topic distribution p(θ|α), word-topic assignment p(z|θ), word p(w|z,φ), topic-word distribution φ_k, and Dirichlet parameters α and β.

Naming warning: one slide caption says "psi, the distribution of words for each topic K" and "phi, the distribution of topics for each document i". The formula slide uses θ for document-topic and φ for topic-word. Go with the formula: **θ = document-topic, φ = topic-word**. α and θ go with documents; β and φ go with topics.

## 3.5 LDA parameters (slide wording)

- **alpha (α)**: Dirichlet prior concentration parameter for **document-topic density**. With higher alpha, documents are assumed to be made of more topics. Controls the **per-document topic distribution**.
- **beta (β)**: the same kind of prior for **topic-word density**. With high beta, topics are assumed to be made of most of the words. Controls the **per-topic word distribution**.
- **theta (θ)**: the distribution of topics over a document, a multinomial distribution.
- Every word is linked to a latent topic Z. Assigning Z to every word creates the topic-word distribution of the corpus.

Short version: **high alpha = documents spread over many topics, low alpha = few topics per document. High beta = topics spread over many words, low beta = few words per topic.**

**Distribution types:** α and β are **Dirichlet** distributions (priors). θ and φ are **multinomial** distributions (the slide groups them this way: Dirichlet over topics and words, multinomial over topics and words).

## 3.6 Probability of a document (joint distribution)

```
P(W, Z, θ, φ ; α, β) = [ product j=1..M of P(θ_j ; α) ]
                     x [ product i=1..K of P(φ_i ; β) ]
                     x [ product t=1..N of P(Z_j,t | θ_j) x P(W_j,t | φ_Z_j,t) ]
```

| Term | What it is | Distribution |
|---|---|---|
| P(θ_j ; α) | topic mix of document j, drawn using α | Dirichlet (one per document, M of them) |
| P(φ_i ; β) | word mix of topic i, drawn using β | Dirichlet (one per topic, K of them) |
| P(Z_j,t \| θ_j) | topic of word t in document j, drawn from the document's topic mix | Multinomial |
| P(W_j,t \| φ_Z) | the actual word, drawn from that topic's word mix | Multinomial |

Where: W = vector of all words in all documents, Z = vector of topics of all words, M = number of documents, K = number of topics, N = number of words.

**Generative story (how LDA imagines a document is written):**
1. For each topic k, draw its word distribution φ_k from Dirichlet(β).
2. For each document, draw its topic mix θ from Dirichlet(α).
3. For each word slot in the document: pick a topic z from θ, then pick a word w from φ_z.

LDA runs this **backwards**: given only the words, it figures out which topics would have produced these documents and which words those topics would produce.

## 3.7 The matrices

- The corpus is written as a **document-term matrix** (document-word matrix). Cell (i, j) = frequency of word W_j in document D_i.
- LDA turns this into two lower-dimensional matrices:
  - **M1 = document-topic matrix**, shape **(N, K)**
  - **M2 = topic-term matrix**, shape **(K, M)**
  - here N = number of documents, K = number of topics, M = vocabulary size.
- Careful: in this slide N and M mean documents and vocabulary size, the opposite of the plate notation. On an exam, say which is which.
- Slide example: 5 documents (central bank, neural network, climate agreement, heart surgery, tech startup), 8 unique words W1 to W8. Document-term matrix is 5 x 8. Document-topic matrix is 5 x K, topic-word matrix is K x 8. (The slide says "5 topics" but draws K1 to K6 and a "6 x 8" topic-word matrix. That is a slide typo. Shape is always K x vocabulary.)
- End goal of LDA: find the best document-topic matrix and topic-word matrix, giving the best document-topic and topic-word distributions.

## 3.8 LDA is iterative (Gibbs sampling idea)

1. **Randomly assign** a topic to every word in every document. Example: D1 = (w1 (k5), w2 (k3), w3 (k1), w4 (k2), w5 (k5), w6 (k4), w7 (k7), w8 (k1)).
2. This gives starting document-topic and topic-word matrices. They are poor, so LDA improves them by sampling.
3. Go through every document D and every word w. For each word, for every topic k, compute:
   - **p1 = p(topic k | document D)** = proportion of words in D currently assigned to topic k.
   - **p2 = p(word w | topic k)** = proportion of the assignments to topic k, over all documents, that come from word w.
4. Reassign w to a new topic k with probability **P = p1 x p2**.
5. Repeat many passes until the assignments settle (converge).

```
p1 = (words in document d assigned to topic k) / (total words in document d)
p2 = (times word w is assigned to topic k across all documents) / (total words assigned to topic k)
P(word w gets topic k) proportional to p1 x p2
```

Standard textbook version with the priors (not on the slides, but this is what the code does):

```
P(z = k) proportional to (n_dk + α) x (n_kw + β) / (n_k + V x β)
   n_dk = words in doc d assigned to topic k
   n_kw = times word w is assigned to topic k in the corpus
   n_k  = total words assigned to topic k
   V    = vocabulary size
```

### Worked mini example of one update (hand computable)

Current assignments, K = 2:

- D1: bank (T1), money (T1), loan (T1), river (T2)
- D2: river (T2), bank (T2), water (T2)
- D3: bank (T1), loan (T1), money (T1)

Update the word **bank in D1**. First remove its own current assignment, so it does not count itself.

- p1 (D1 without that bank: money T1, loan T1, river T2): p1(T1) = 2/3, p1(T2) = 1/3.
- Topic totals without it: T1 has money, loan (D1), bank, loan, money (D3) = 5 words, of which bank = 1. T2 has river (D1), river, bank, water (D2) = 4 words, of which bank = 1.
- p2(T1) = 1/5 = 0.20. p2(T2) = 1/4 = 0.25.
- T1: 2/3 x 0.20 = 0.133. T2: 1/3 x 0.25 = 0.083.
- Normalize: T1 = 0.133 / 0.217 = 0.615, T2 = 0.385. bank in D1 will most likely be assigned T1.

## 3.9 LDA process and implementation

**LDA process (4 boxes):** preprocess -> train -> score -> evaluate.

**LDA implementation steps:**
1. Loading data
2. Data cleaning
3. Exploratory analysis
4. Preparing data for LDA analysis
5. LDA model training
6. Analyzing LDA model results

**pyLDAvis** (visualization package for LDA results):
- Helps interpret individual topics: select each topic to view its most frequent and/or most "relevant" terms, using different values of the **lambda (λ)** parameter.
- Helps see how topics relate: the **Intertopic Distance Plot** shows how topics relate to each other, including higher-level groups of topics. Overlapping bubbles mean topics are too similar (often a sign of too many topics). Bubble size = how common the topic is.

## 3.10 Evaluating topic models

**Approaches:**
1. **Eye balling models**: look at the top N words, and at topics vs documents.
2. **Intrinsic evaluation metrics**: capture model semantics, topic interpretability.
3. **Human judgments**: what is the topic?
4. **Extrinsic evaluation metrics** (as grouped on the slide):
   - **Topic coherence**: scores a single topic by measuring the semantic similarity between its high-scoring words. Higher is better.
   - **Perplexity**: how well the model predicts unseen (held-out) documents. **Lower perplexity is better**, since the model predicts unseen words better and understands the topics better.

Formulas (not on the slides, know them in case):

```
Perplexity = exp( - [sum over documents of log p(w_d)] / [total number of words] )
   lower = better

NPMI(a, b) = log( P(a,b) / (P(a) x P(b)) ) / ( -log P(a,b) )
   P(a), P(b) = share of documents containing word a, word b
   P(a,b)     = share of documents containing both
   range -1 (never together) to 0 (independent) to 1 (always together)
   topic coherence = average NPMI over all pairs of the topic's top words. Higher = better.
```

## 3.11 Choosing the number of topics K

- Fit LDA for a range of K (e.g. 3 to 8). Score each with coherence (and perplexity).
- Pick the K with the **highest coherence**, or the elbow where coherence stops improving. Lower perplexity also helps, but perplexity often keeps improving as K grows while topics get less readable, so coherence is usually the deciding metric.
- Check by eye that the top words make sense, and in pyLDAvis that the bubbles do not overlap.
- LDA results shift with the random start, so run several seeds per K and average (I did 5 seeds in the summative).

## 3.12 LDA vs LSA

- **LSA**: decomposes the TF-IDF (or count) matrix with **SVD** (singular value decomposition, a matrix factorization) into a few main directions. Words can get **negative weights**, which makes topics harder to read.
- **LDA**: probabilistic, all weights are probabilities (0 to 1), so a topic is a clean word distribution. LDA usually runs on raw counts, not TF-IDF.

---

# 4. Sequential Pattern Mining (SPM)

## 4.1 What SPM is

- A popular data mining task introduced in **1994 by Agrawal and Srikant**.
- The process of finding frequently occurring **ordered** events or subsequences as patterns.
- Goal: find all subsequences that appear frequently in a set of discrete sequences.
- A **sequence** is an ordered list of symbols, formally the ordered set {s1, s2, ..., sn}.

## 4.2 Sequential rules vs association rules

- **Association rules** find co-occurrence relationships (associations) among items in records (transactions) whose items are **not ordered**. Time order is ignored.
- **Sequential rules** are found in **sequences**, where order matters.
- Some sequential rule algorithms, like **CMRules**, first find association rules, then filter out the ones that break the sequential order.
- Why order matters:
  - Market basket: did people buy a bed first, then bed sheets later?
  - NLP / text mining: word order in a sentence is vital for language patterns. Association rules are not appropriate there.
- Example sequential pattern: "5% of customers buy bed first, then mattress and then pillows." The items are bought one after another, not at the same time.

## 4.3 Types of SPM

- **String mining**: data uses a limited alphabet. DNA (A, T, C, G) or ASCII character sequences.
- **Itemset mining**: finding patterns in an ordered data set. Used in marketing and sales: co-purchases, cross-promoting products, managing inventory, setting price levels.

## 4.4 Items, itemsets, sequences

- **I**: the set of all items (symbols). Example I = {a, b, c, d, e}.
- **Itemset**: a subset of I. {a, b, c} has 3 items. {d, e} has 2. **An itemset cannot contain the same item twice.**
- Notations seen in class for "a and b together": {ab}, {a,b}, {(ab)}.
- **Sequence**: an ordered list of itemsets S = [X1, X2, ..., Xn], where each Xj is a subset of I.
  - {(a,b), (c)}: a and b bought at the same time, then c later.
  - {(a), (a), (c)}: a, then a again, then c. The same item can repeat **across** itemsets.
- **Order rules**:
  - {a, b} (separate events) is **not** equal to {b, a}. Order between itemsets matters.
  - {(a,b)} **is** equal to {(b,a)}. Inside one itemset there is no order. Write items in alphabetical order inside an itemset.
- **Length of a sequence** = total number of items (count every item in every itemset). <(a),(f,g)> has length 3. In GSP a "k-sequence" means k items, not k itemsets.

## 4.5 Subsequence (containment)

SA = [A1, A2, ..., Ar] is a **subsequence** of SB = [B1, B2, ..., Bt] (written SA ⊑ SB) if and only if there are integers 1 <= i1 < i2 < ... < ir <= t such that A1 ⊆ B_i1, A2 ⊆ B_i2, ..., Ar ⊆ B_ir.

In plain words: each itemset of SA must fit **inside** some itemset of SB, using **different** itemsets of SB, **in the same order**. Items do not need to be consecutive.

Slide examples:

| Check | Answer | Why |
|---|---|---|
| {(a,c)} ⊑ {(a,b,c)} | Yes | a and c are both inside the one itemset (a,b,c) |
| {(a,c)} ⊑ {(a),(c)} | **No** | a and c must be together in one itemset, but SB has them apart |
| {(a),(c)} ⊑ {(a,b),(d),(b,c)} | Yes | (a) fits in (a,b), (c) fits in the later (b,c) |
| {(a),(c)} ⊑ {(a,c),(d)} | **No** | a and c are in the same itemset of SB, but SA needs c strictly after a |

Plain example: {a, b, c} and {y, e} are subsequences of {a, b, g, q, y, e, c}.

**Sequence pattern**: a subsequence found in many sequences. When its support is at least the minimum support, it is a sequential pattern. Slide example: the pattern <a, b> is mined from {b, x, c, a}, {a, b, q} and {a, u, b}. Careful, it is only contained in the last two (in the first one, b comes before a), so its support is 2.

## 4.6 Support of a sequence

```
support(SA) = number of sequences in database D that contain SA (as a subsequence)
relative support = support / |D|
A sequence is frequent if support >= min_sup.
```

Count each sequence **once**, even if the pattern appears several times inside it.

Slide example:

| SID | Sequence |
|---|---|
| S1 | {(a,b), (c), (a)} |
| S2 | {(a), (b), (c)} |
| S3 | {(b), (c), (d)} |
| S4 | {(b), (a,b), (c)} |

- Sup{a} = 3 (S1, S2, S4)
- Sup{b} = 4 (S1, S2, S3, S4)
- Sup{(a,b)} = 2 (S1, S4), a and b in the same itemset
- Sup{(a),(b)} = ? on the slide. **Answer: 1 (only S2).** S1: b is never after a's itemset (the b is in the same itemset as the first a, and the later a has nothing after it). S4: a is in (a,b), and nothing after it has b. Same-itemset does not count as "after".

## 4.7 Methods for SPM

- **Apriori-based approaches**
  - **GSP (Generalized Sequential Pattern)**: works on **horizontal** data (each row is a sequence ID with its full sequence).
  - **SPADE**: works on **vertical** data (each item with the list of where it appears).
- **Pattern-growth-based approaches**
  - **FreeSpan**
  - **PrefixSpan**
- Slide chart (runtime vs database size): PrefixSpan is fastest, then SPADE, then FreeSpan, with GSP slowest.

## 4.8 The Apriori idea behind GSP

- **Level-wise search**: k-sequences are used to explore (k+1)-sequences.
- **Apriori property (anti-monotone)**: if a sequence S is not frequent, none of its super-sequences can be frequent. Example: <hb> infrequent, so <hab> and <(ah)b> are infrequent too.
- Two steps each level:
  1. **Join step**: generate the candidates C_k by joining L_(k-1) with itself.
  2. **Prune step**: C_k is a superset of L_k (it holds every frequent k-sequence plus some that are not). Use the Apriori property to throw out any candidate that has an infrequent (k-1)-subsequence before counting.
- Then **count support** for the remaining candidates by scanning the database, keep those with support >= min_sup, and repeat until no new frequent sequences.

## 4.9 The GSP join rule

**Core rule: the suffix of S1 (drop its first item) must equal the prefix of S2 (drop its last item).**

Then the candidate is S1 extended by the last item of S2:
- If S2's last item was a **separate event** (its own itemset), append it as a new itemset.
- If S2's last item was **in the same event** as the item before it, add it into S1's last itemset.

Slide examples:

| S1 | S2 | Suffix of S1 | Prefix of S2 | Match? | Candidate |
|---|---|---|---|---|---|
| {a, (b,f)} | {(b,f), c} | {(b,f)} | {(b,f)} | Yes | {a, (b,f), c} (c appended as a separate event) |
| {a, (b,f)} | {(b,f,c)} | {(b,f)} | {(b,f)} | Yes | {a, (b,f,c)} (c added to the same event) |
| {a, (b,f)} | {b, f, c} | {(b,f)} | {b, f} | **No** | none. Same items, different structure: (b,f) is one event, b, f are two events. |

**Level 1 to level 2 is special** (every 1-sequence has an empty suffix/prefix). From the frequent single items, generate:
- every ordered pair as separate events <(x),(y)>, and
- every unordered pair in one event <(x,y)> with x < y.

The slides generate <(x),(y)> only for x ≠ y. Strict GSP also generates <(x),(x)> (the same item twice in a row). With 6 frequent items: 30 (x ≠ y) + 6 (x = x) + 15 (x,y) = 51 candidates. The slides use 30 + 15 = 45. In this example every <(x),(x)> has support 1, so the result is the same either way. If asked, mention it.

## 4.10 Fully worked GSP example (the slide database)

Database, **min_sup = 2**:

| SID | Sequence |
|---|---|
| S1 | {(a,b), (c), (f,g), (g), (e)} |
| S2 | {(a,d), (c), (b), (a,b,e,f)} |
| S3 | {(a), (b), (f,g), (e)} |
| S4 | {(b), (f,g)} |

### Level 1: count single items

| Item | Support | Appears in | Frequent? |
|---|---|---|---|
| a | 3 | S1, S2, S3 | yes |
| b | 4 | S1, S2, S3, S4 | yes |
| c | 2 | S1, S2 | yes |
| d | 1 | S2 | **no, removed** |
| e | 3 | S1, S2, S3 | yes |
| f | 4 | S1, S2, S3, S4 | yes |
| g | 3 | S1, S3, S4 | yes |

L1 = {a, b, c, e, f, g}. Drop d from everything from now on.

### Level 2: generate and count

Candidates: <(x),(y)> for all ordered pairs and <(x,y)> for all unordered pairs of {a, b, c, e, f, g}. No pruning is possible at this level (all 1-subsequences are frequent).

Full support table (slide values, checked):

| <(x),(y)> | sup | <(x),(y)> | sup | <(x),(y)> | sup |
|---|---|---|---|---|---|
| (a),(b) | **2** | (b),(a) | 1 | (c),(a) | 1 |
| (a),(c) | **2** | (b),(c) | 1 | (c),(b) | 1 |
| (a),(e) | **3** | (b),(e) | **3** | (c),(e) | **2** |
| (a),(f) | **3** | (b),(f) | **4** | (c),(f) | **2** |
| (a),(g) | **2** | (b),(g) | **3** | (c),(g) | 1 |
| (e),(a) | 0 | (f),(a) | 0 | (g),(a) | 0 |
| (e),(b) | 0 | (f),(b) | 0 | (g),(b) | 0 |
| (e),(c) | 0 | (f),(c) | 0 | (g),(c) | 0 |
| (e),(f) | 0 | (f),(e) | **2** | (g),(e) | **2** |
| (e),(g) | 0 | (f),(g) | 0 | (g),(f) | 0 |

| <(x,y)> | sup | <(x,y)> | sup | <(x,y)> | sup |
|---|---|---|---|---|---|
| (a,b) | **2** | (b,c) | 0 | (c,e) | 0 |
| (a,c) | 0 | (b,e) | 1 | (c,f) | 0 |
| (a,e) | 1 | (b,f) | 1 | (c,g) | 0 |
| (a,f) | 1 | (b,g) | 0 | (e,f) | 1 |
| (a,g) | 0 | | | (e,g) | 0 |
| | | | | (f,g) | **3** |

How to count a tricky one, <(f),(e)>: S1 has f in (f,g) and e later in (e), yes. S2 has f and e only in the same itemset (a,b,e,f), so **no**. S3 yes. S4 has no e. Support = 2.

How to count <(a),(b)>: S1 has a and b only in the same itemset (a,b), no later b, so **no**. S2: a in (a,d), b later in (b), yes. S3 yes. S4 no a. Support = 2.

**L2 (14 frequent 2-sequences):**

| 2-sequence | sup | 2-sequence | sup |
|---|---|---|---|
| (a),(b) | 2 | (b),(f) | 4 |
| (a),(c) | 2 | (b),(g) | 3 |
| (a),(e) | 3 | (c),(e) | 2 |
| (a),(f) | 3 | (c),(f) | 2 |
| (a),(g) | 2 | (f),(e) | 2 |
| (a,b) | 2 | (f,g) | 3 |
| (b),(e) | 3 | (g),(e) | 2 |

### Level 3: join, prune, count

Join every pair where (S1 without its first item) = (S2 without its last item). For 2-sequences, this means the last item of S1 is the first item of S2.

| Join | Candidate | Prune check (every 2-subsequence frequent?) | Support | Frequent? |
|---|---|---|---|---|
| (a),(b) + (b),(e) | (a),(b),(e) | (a)(b), (a)(e), (b)(e) all frequent | 2 (S2, S3) | yes |
| (a),(b) + (b),(f) | (a),(b),(f) | ok | 2 (S2, S3) | yes |
| (a),(b) + (b),(g) | (a),(b),(g) | ok | 1 (S3) | no |
| (a),(c) + (c),(e) | (a),(c),(e) | ok | 2 (S1, S2) | yes |
| (a),(c) + (c),(f) | (a),(c),(f) | ok | 2 (S1, S2) | yes |
| (a),(f) + (f),(e) | (a),(f),(e) | ok | 2 (S1, S3) | yes |
| (a),(f) + (f,g) | (a),(f,g) | (a)(f), (a)(g), (f,g) ok | 2 (S1, S3) | yes |
| (a),(g) + (g),(e) | (a),(g),(e) | ok | 2 (S1, S3) | yes |
| (a,b) + (b),(e) | (a,b),(e) | (a,b), (a)(e), (b)(e) ok | 1 (S1) | no |
| (a,b) + (b),(f) | (a,b),(f) | ok | 1 (S1) | no |
| (a,b) + (b),(g) | (a,b),(g) | ok | 1 (S1) | no |
| (b),(f) + (f),(e) | (b),(f),(e) | ok | 2 (S1, S3) | yes |
| (b),(f) + (f,g) | (b),(f,g) | ok | 3 (S1, S3, S4) | yes |
| (b),(g) + (g),(e) | (b),(g),(e) | ok | 2 (S1, S3) | yes |
| (c),(f) + (f),(e) | (c),(f),(e) | ok | 1 (S1) | no |
| (c),(f) + (f,g) | (c),(f,g) | **(c),(g) has sup 1, not frequent** | pruned, never counted | no |
| (f,g) + (g),(e) | (f,g),(e) | (f,g), (f)(e), (g)(e) ok | 2 (S1, S3) | yes |

Notes on the joins:
- (a),(f) + (f,g): suffix of (a),(f) is (f). Prefix of (f,g) after dropping g is (f). Match. g was in the same event as f in S2, so g joins f's itemset: (a),(f,g).
- (f,g) + (g),(e): suffix of (f,g) after dropping f is (g). Prefix of (g),(e) is (g). Match. e was a separate event, so append: (f,g),(e).
- Why (a),(b),(g) fails: S2 has no g. S3 has a, b, then (f,g). Only S3.
- Why (c),(f),(e) fails: S2's f and e are in the same itemset.

**L3 (11 frequent 3-sequences):**

| 3-sequence | sup |
|---|---|
| (a),(b),(e) | 2 |
| (a),(b),(f) | 2 |
| (a),(c),(e) | 2 |
| (a),(c),(f) | 2 |
| (a),(f),(e) | 2 |
| (a),(f,g) | 2 |
| (a),(g),(e) | 2 |
| (b),(f),(e) | 2 |
| (b),(f,g) | 3 |
| (b),(g),(e) | 2 |
| (f,g),(e) | 2 |

**Slide discrepancy, read this.** The slides list (f,g),(e) as a candidate but then leave it out of the counted table and the final list, so the slide's L3 has only 10 rows. It is frequent (S1 and S3 both have f and g together, then e later), so it belongs in L3. The slides' own "sequential rules" slide later uses <(a),(f,g),(e)> and <(b),(f,g),(e)> as patterns, and those only come from (f,g),(e). If the exam reuses this database, include (f,g),(e) and write its support.

The slides also wrote the (c),(f) + (f,g) join result as "(c),(f),(g)" with support 1. The correct join result is (c),(f,g) (g stays in f's itemset), and it gets pruned because (c),(g) is infrequent. Either way it is not frequent.

### Level 4

| Join | Candidate | Prune check (every 3-subsequence in L3?) | Support | Frequent? |
|---|---|---|---|---|
| (a),(b),(f) + (b),(f),(e) | (a),(b),(f),(e) | (b)(f)(e), (a)(f)(e), (a)(b)(e), (a)(b)(f) all in L3 | 1 (S3 only; S2 has f and e together) | no |
| (a),(b),(f) + (b),(f,g) | (a),(b),(f,g) | **(a),(b),(g) not in L3** | pruned | no |
| (a),(f,g) + (f,g),(e) | **(a),(f,g),(e)** | (f,g)(e), (a)(g)(e), (a)(f)(e), (a)(f,g) all in L3 | 2 (S1, S3) | **yes** |
| (b),(f,g) + (f,g),(e) | **(b),(f,g),(e)** | (f,g)(e), (b)(g)(e), (b)(f)(e), (b)(f,g) all in L3 | 2 (S1, S3) | **yes** |

The slides stop at (a),(b),(f),(e) with support 1 and say nothing is frequent at length 4, because they dropped (f,g),(e). With it included, L4 = {<(a),(f,g),(e)> : 2, <(b),(f,g),(e)> : 2}, which matches the patterns on the sequential rules slide.

### Level 5

(a),(f,g),(e) and (b),(f,g),(e) cannot join (suffix (f,g),(e) of one vs prefix (b),(f,g) of the other do not match). No candidates. **Stop.**

### Final answer for this database (min_sup = 2)

- L1: a 3, b 4, c 2, e 3, f 4, g 3
- L2: the 14 sequences above
- L3: the 11 sequences above (slides show 10, missing (f,g),(e))
- L4: (a),(f,g),(e) 2 and (b),(f,g),(e) 2

## 4.11 GSP procedure checklist (for any exam database)

1. Write the database. Note min_sup (convert a percent to a count: min_sup% x number of sequences, round up).
2. Count every single item (once per sequence). Keep the frequent ones (L1).
3. Build all 2-candidates: <(x),(y)> for every ordered pair and <(x,y)> for every unordered pair. Count. Keep L2.
4. For k >= 3: join pairs where S1 minus first item = S2 minus last item. Add S2's last item as a new itemset if it was separate in S2, or into the last itemset if it was together in S2.
5. Prune: for each candidate, drop one item at a time. Every result must be in L(k-1). If not, delete the candidate without counting.
6. Count support of survivors. Keep L_k.
7. Repeat until there are no candidates. List every L_k with supports.

## 4.12 Common GSP mistakes

- Counting a sequence more than once for one pattern. Support counts sequences, not occurrences.
- Treating items in the **same itemset** as ordered. (a,b) means at the same time, so it does not give <(a),(b)> or <(b),(a)>.
- Requiring items to be consecutive. They only need to keep their order.
- Letting two pattern itemsets map into the **same** database itemset. Each pattern itemset needs its own itemset, in order.
- Forgetting <(x,y)> candidates at level 2, or forgetting that (a,b) = (b,a).
- Joining structures that do not match: {(b,f)} vs {b, f} is not a match.
- Skipping the prune step. A candidate with an infrequent subsequence is deleted before counting.
- Measuring length by number of itemsets. Length counts items: <(a),(f,g)> is a 3-sequence.

## 4.13 SPADE (vertical format)

SPADE (Sequential PAttern Discovery using Equivalence classes) stores the data **vertically**: for each item, the list of (SID, EID) pairs where it occurs. SID is the sequence ID and EID is the position (itemset number) inside the sequence. Support = number of distinct SIDs in the list. Longer patterns are found by **joining id-lists**, so the database is scanned only a few times. The slides only name it, so the example below is mine.

Same database, EID = itemset number:

| Item | (SID, EID) list | Support |
|---|---|---|
| a | (1,1) (2,1) (2,4) (3,1) | 3 |
| b | (1,1) (2,3) (2,4) (3,2) (4,1) | 4 |

- <(a,b)> (same itemset): same SID and **same** EID. (1,1) and (2,4). SIDs {1, 2}. Support 2.
- <(a),(b)> (b after a): same SID and b's EID **greater** than a's. S1: a at 1, b at 1, no. S2: a at 1, b at 3, yes. S3: a at 1, b at 2, yes. Support 2.

Both match GSP.

## 4.14 PrefixSpan (pattern growth)

PrefixSpan does not generate candidates. It picks a frequent prefix, builds a **projected database** (for each sequence, the part after the first occurrence of the prefix), finds the frequent items in that projected database, and grows the prefix with them. It repeats on smaller and smaller projections. That is why it is the fastest on the slide chart. FreeSpan is the earlier pattern-growth method that PrefixSpan improved.

Mini example, prefix <a> on the same database:

| SID | <a>-projected sequence |
|---|---|
| S1 | (_b), (c), (f,g), (g), (e) |
| S2 | (_d), (c), (b), (a,b,e,f) |
| S3 | (b), (f,g), (e) |
| S4 | (none, no a) |

The underscore means "in the same itemset as the prefix". Count in the projection: b as a separate event in S2, S3 gives <(a),(b)> = 2. _b in S1 (and the later (a,b,e,f) itemset in S2) gives <(a,b)> = 2. c in S1, S2 gives <(a),(c)> = 2. e in S1, S2, S3 gives <(a),(e)> = 3. Same results as GSP, with no candidate generation.

## 4.15 Sequential rules

Sequential rules are built by finding sequential patterns and then combining them into rules. Two kinds:

### 1) Standard sequential rule

- Form **X -> Y**, where X and Y are **sequential patterns** (order matters inside X and inside Y). Example: <{a}, {b,c}> -> <{d}, {e}>.
- X is the **antecedent** (left side), Y is the **consequent** (right side).
- Two user thresholds: minimum support > 0 and minimum confidence > 0.

```
support(X -> Y)    = (number of sequences containing the whole rule, X then Y) / (total sequences)
confidence(X -> Y) = (number of sequences containing X then Y) / (number of sequences containing X)
```

Slide example:

| SID | Sequence |
|---|---|
| 1 | {(a), (a,b,c), (a,c), (d), (c,f)} |
| 2 | {(a,d), (c), (b,c), (a,e)} |
| 3 | {(e,f), (a,b), (d,f), (c), (b)} |
| 4 | {(e), (g), (a,f), (c), (b), (c)} |

| Rule | Support | Confidence |
|---|---|---|
| <{a},{b}> -> <{f}> | 1/4 = 25% | 1/4 = 25% |
| <{b},{a}> -> <{f}> | 1/4 = 25% | 1/2 = 50% |

Why: <{a},{b}> (a, then b later) is in all 4 sequences, but f follows only in S1, so confidence = 1/4. <{b},{a}> is only in S1 (b in (a,b,c), a in (a,c)) and S2 (b in (b,c), a in (a,e)), and f follows only in S1, so confidence = 1/2.
- min support 25%, min confidence 50%: only rule 2 is a sequential rule.
- min support 25%, min confidence 25%: both are.

### 2) Partially-ordered sequential rule

- Form **X -> Y**, where X and Y are **itemsets that are unordered** inside. X and Y must be **disjoint** (no shared items) and **non-empty**.
- Example {a,b} -> {f}: if we see a and b (in any order), they will be followed by f.
- Usually more interesting because one rule summarizes many standard rules. Slide: the six orderings of {Vivaldi, Mozart, Handel} -> {Berlioz} collapse into the single rule {Mozart, Vivaldi, Handel} -> {Berlioz}.

```
support(X -> Y)    = (sequences where all of X occurs before all of Y) / (total sequences)
confidence(X -> Y) = (sequences where all of X occurs before all of Y) / (sequences that contain all of X)
```

Task (Fournier-Viger, 2010): find all valid rules, those with support >= minSup and confidence >= minConf.

Slide example database (each {} is one itemset):

| ID | Sequence |
|---|---|
| seq1 | {a,b}, {c}, {f}, {g}, {e} |
| seq2 | {a,d}, {c}, {b}, {a,b,e,f} |
| seq3 | {a}, {b}, {f}, {e} |
| seq4 | {b}, {f,g} |

Rules found (minSup = 0.5, minConf = 0.5):

| ID | Rule | Support | Confidence |
|---|---|---|---|
| r1 | {a,b,c} -> {e} | 0.5 | 1.0 |
| r2 | {a} -> {c,e,f} | 0.5 | 0.66 |
| r3 | {a,b} -> {e,f} | 0.75 | 1.0 |
| r4 | {b} -> {e,f} | 0.75 | 0.75 |
| r5 | {a} -> {e,f} | 0.75 | 1.0 |
| r6 | {c} -> {f} | 0.5 | 1.0 |
| r7 | {a} -> {b} | 0.5 | 0.66 |

- If minSup = 0.5 and minConf = 0.75, only r1, r3, r4, r5, r6 remain.
- Check r2: a occurs before c, e and f in seq1 and seq2. a occurs in seq1, seq2, seq3. Support 2/4 = 0.5, confidence 2/3 = 0.66.
- Check r4: b before e and f in seq1, seq2 (b in {b}, then {a,b,e,f}), seq3. b occurs in all 4. Support 3/4, confidence 3/4.

### Rules from the GSP database (last slide)

Patterns: <(a),(f,g),(e)> and <(b),(f,g),(e)>, each support 2. Min sup 50%, min confidence 50%. Split each pattern into antecedent and consequent:

| Rule | Support | Confidence |
|---|---|---|
| {(a)} -> {(f,g),(e)} | 2/4 = 50% | sup(whole) / sup(a) = 2/3 = 67% |
| {(a),(f,g)} -> {(e)} | 2/4 = 50% | 2 / sup((a),(f,g)) = 2/2 = 100% |
| {(b)} -> {(f,g),(e)} | 2/4 = 50% | 2 / sup(b) = 2/4 = 50% |
| {(b),(f,g)} -> {(e)} | 2/4 = 50% | 2 / sup((b),(f,g)) = 2/3 = 67% |

All four pass 50% / 50%.

**Method**: rule support = support of the whole pattern / number of sequences. Confidence = support of the whole pattern / support of the antecedent alone. Pull the antecedent supports from the GSP tables (a = 3, b = 4, (a),(f,g) = 2, (b),(f,g) = 3).

---

# 5. Proximity Measures Used in the Midterm

Quick refresher only (prelim topic), because the summative used them on text.

```
Cosine similarity:   cos(A, B) = (A . B) / (||A|| x ||B||)
   A . B = sum of a_i x b_i      ||A|| = sqrt(sum of a_i^2)
   1 = same word mix, 0 = no shared words (for non-negative vectors like TF-IDF)

Jaccard similarity:  J(A, B) = |A ∩ B| / |A ∪ B|
   shared items / all distinct items in either set

Euclidean distance:  d(A, B) = sqrt( sum of (a_i - b_i)^2 )
   smaller = closer
```

**Worked example on the TF-IDF matrix from Section 1.11 (D1 vs D2):**

- D1 = (0.0795, 0, 0, 0.0795, 0.0293, 0.0293), D2 = (0, 0, 0, 0, 0.0352, 0.0352)
- Dot product = 0.0293 x 0.0352 + 0.0293 x 0.0352 = 0.00206
- ||D1|| = sqrt(0.0795^2 + 0.0795^2 + 0.0293^2 + 0.0293^2) = sqrt(0.014357) = 0.1198
- ||D2|| = sqrt(2 x 0.0352^2) = 0.0498
- cos = 0.00206 / (0.1198 x 0.0498) = **0.346**
- Jaccard on term sets: D1 = {quick, brown, fox, jumped, lazy, dog}, D2 = {brown, dog, fox, lazy}. Shared 4, union 6. J = **0.667**.
- Euclidean: sqrt(0.0795^2 + 0.0795^2 + (0.0293 - 0.0352)^2 x 2) = **0.113**

Why cosine for text: it measures the direction (word mix) and ignores document length.

---

# 6. Lab Activities

## 6.1 Midterm Lec Activity: sentiment analysis steps by hand

**What it asked**: present the steps of sentiment analysis using (a) VADER, (b) TextBlob, (c) SentiWordNet, and show the manual classification process with a simple table of data. Handwritten.

I used four sentences: "I love this phone", "The battery is bad", "The food is not good", "The class starts at 8".

### VADER steps

1. Split the sentence into words.
2. Look up each word's score in the VADER lexicon (-4 very negative to +4 very positive). Unknown words = 0.
3. Adjust: "not" before a word flips it (score x -0.74), "very" boosts, "!" and ALL CAPS add emphasis.
4. Sum the scores.
5. Normalize: `compound = sum / sqrt(sum^2 + 15)`.
6. Classify: >= 0.05 positive, <= -0.05 negative, otherwise neutral.

| Sentence | Scores | Sum | Compound | Result |
|---|---|---|---|---|
| I love this phone | love 3.2 | 3.2 | 3.2 / sqrt(10.24 + 15) = 0.637 | Positive |
| The battery is bad | bad -2.5 | -2.5 | -2.5 / sqrt(6.25 + 15) = -0.542 | Negative |
| The food is not good | good 1.9 x -0.74 = -1.406 | -1.406 | -1.406 / sqrt(1.977 + 15) = -0.341 | Negative |
| The class starts at 8 | none | 0 | 0 | Neutral |

### TextBlob steps

1. Split into words.
2. Look up each word's polarity (-1 to +1) and subjectivity (0 to 1) in TextBlob's lexicon (mostly adjectives).
3. Modifiers: "not" multiplies polarity by -0.5, "very" multiplies by 1.3.
4. Average the polarity of the words found.
5. Classify: > 0 positive, < 0 negative, = 0 neutral.

| Sentence | Polarity | Subjectivity | Result |
|---|---|---|---|
| I love this phone | love 0.5 -> 0.5 | 0.6 | Positive |
| The battery is bad | bad -0.7 -> -0.7 | 0.67 | Negative |
| The food is not good | good 0.7 x -0.5 = -0.35 | 0.6 | Negative |
| The class starts at 8 | 0 | 0 | Neutral |

### SentiWordNet steps

1. Split into words.
2. POS-tag each word (noun, verb, adjective, adverb).
3. Lemmatize (starts -> start).
4. Look up the word's most common meaning (first synset) and take its positive and negative scores.
5. If "not" comes before a word, swap its positive and negative scores.
6. Sum all positive and all negative scores.
7. Classify: bigger total wins, equal = neutral.

| Sentence | (pos, neg) | Total pos | Total neg | Result |
|---|---|---|---|---|
| I love this phone | love (0.5, 0) | 0.5 | 0 | Positive |
| The battery is bad | bad (0, 0.625) | 0 | 0.625 | Negative |
| The food is not good | good (0.75, 0) swapped -> (0, 0.75) | 0 | 0.75 | Negative |
| The class starts at 8 | all 0 | 0 | 0 | Neutral |

**Remember the differences**: VADER gives one compound score and handles slang, caps and punctuation (social media). TextBlob gives polarity plus subjectivity. SentiWordNet keeps positive and negative as separate totals and needs POS tagging because scores depend on word meaning.

## 6.2 Midterm Activity 2: Time Series Forecasting

**What it asked** (pairs, with Tay-og):
1. Explore and preprocess (missing values, stationarity testing).
2. Build two models, ARIMA/SARIMAX and XGBoost (or similar), for the average temperature of the Philippines:
   - (a) trained on 1853-2011
   - (b) trained on 2009-2011
   - both validated on **2012**
   - compare the models across the two periods, with training and validation metrics, and discuss.
3. Script / output / remarks table. 4. Plots of actual vs predicted.

There are no time series slides in my midterm notes folder, so the concepts below come from the activity.

### Method steps

1. **Load and fix dates.** The file switched date formats in 1904 (1903-12-01 to 01/01/04). Two-digit years would be read as 2004, so I rebuilt a clean monthly date range from January 1853 (1920 months).
2. **Missing values.** 38 missing months, mostly one 33-month gap (April 1862 to December 1864). Filled each missing month with the average of the **same calendar month** from the 10 years before and after. A straight line across the gap would erase three seasons.
3. **Explore.** Yearly averages show about 1 degree of warming, mostly after 1920.
4. **Decomposition** (additive): `y(t) = trend(t) + seasonal(t) + residual(t)`. Seasonal swing about 2.7 degrees, repeating every 12 months. (Multiplicative version: `y = trend x seasonal x residual`, used when the swing grows with the level.)
5. **Stationarity tests.** A **stationary** series has a mean and spread that do not change over time. ARIMA-type models assume it.
   - **ADF** (Augmented Dickey-Fuller): null hypothesis = **not stationary**. p < 0.05 -> stationary.
   - **KPSS**: null hypothesis = **stationary**. p < 0.05 -> not stationary.
   - The two come from opposite sides. Call it stationary when ADF p < 0.05 **and** KPSS p >= 0.05.
   - 1853-2011 as is: ADF 0.1058, KPSS 0.01 -> not stationary (the trend). After **seasonal differencing** (value minus same month last year, `y(t) - y(t-12)`): ADF about 0, KPSS 0.1 -> stationary.
   - 2009-2011: ADF never passed, too few points (36 months, 24 after differencing). Treated as inconclusive.
6. **ACF / PACF.** ACF = correlation with each earlier lag. PACF = the same with the shorter lags taken out. A big ACF spike at lag 12 -> one seasonal MA term. PACF cutting off after lag 1 or 2 -> small AR order.
7. **Split.** Train (a) 1908 months, train (b) 36 months, validation = 12 months of 2012.
8. **SARIMAX.** `SARIMAX(p, d, q)(P, D, Q, s)`:
   - p = AR order (how many past values), d = differencing, q = MA order (how many past errors)
   - P, D, Q = the same for the seasonal part, s = season length (12 for monthly)
   - Fixed d = 0 and D = 1, added a constant (trend "c") for the warming. Grid search over p, q in 0 to 2 and P, Q in 0 to 1, ranked by **AIC**.
   - `AIC = 2k - 2 ln(L)` (k = number of parameters, L = likelihood). Lower is better. It rewards fit and charges for extra parameters. AIC values from different data lengths cannot be compared.
   - Best (a): **(1, 0, 2)(0, 1, 1, 12)**, AIC 1028.45. Seasonal MA -0.94. Constant gives about 0.8 degrees per century.
   - Best (b): (1, 0, 0)(1, 1, 0, 12), AIC 28.93. Its trend term was not significant (p = 0.64), three years is too short to see warming.
9. **XGBoost.** Gradient boosted trees: hundreds of small trees, each fixing the errors of the ones before. It has no sense of time, so I gave it lag features: month, lag_12, lag_24, and the 12-month average ending a year earlier. Every feature is at least 12 months old, so all of 2012 can be predicted from data up to 2011. Period (b) used only month and lag_12 (too few rows). 300 trees, depth 3, learning rate 0.05. lag_12 carried about two thirds of the importance.
10. **Evaluate** on training and on 2012, with MAE, RMSE, MAPE and R2.

### Metrics (memorize)

```
MAE  = (1/n) x sum of |y_i - ŷ_i|                       average miss, in the data's units
RMSE = sqrt( (1/n) x sum of (y_i - ŷ_i)^2 )             punishes big misses more; RMSE >= MAE
MAPE = (100/n) x sum of |(y_i - ŷ_i) / y_i|             average miss as a percent
R2   = 1 - [sum of (y_i - ŷ_i)^2] / [sum of (y_i - ȳ)^2] share of variation explained; 1 perfect, 0 = no better than the mean
   y_i = actual, ŷ_i = predicted, ȳ = mean of the actual values, n = number of points
```

### Results to remember

| Model | Data | MAE | RMSE | MAPE % | R2 |
|---|---|---|---|---|---|
| SARIMAX a | validation | 0.183 | 0.239 | 0.680 | 0.856 |
| SARIMAX b | validation | 0.224 | 0.288 | 0.830 | 0.792 |
| XGBoost a | validation | 0.298 | 0.356 | 1.105 | 0.682 |
| XGBoost b | training | 0.028 | 0.032 | 0.104 | **0.999** |
| XGBoost b | validation | 0.248 | 0.296 | 0.912 | 0.780 |

**Takeaways for exam questions:**
- **Best model: SARIMAX on 1853-2011.** Longer history lets it estimate the trend and a stable seasonal pattern.
- **XGBoost b overfit**: R2 0.999 in training, 0.780 in validation. With 24 rows it memorized 2010 and 2011. A big training/validation gap = overfitting.
- Trees **cannot extrapolate a trend**. They only predict values inside the range seen in training. SARIMAX has the trend built in through its constant.
- Every model missed December and January 2012 (warmer than history suggested). Year-to-year climate events like El Niño are not in a temperature-only model. Adding them as an exogenous input (the X in SARIMAX) is the fix.
- Training and validation are scored differently. SARIMAX training uses one-step-ahead predictions (the real previous month is known). Validation forecasts all 12 months at once.

## 6.3 Midterm Summative Activity: topic modeling, proximity, rule-based sentiment

**What it asked** (group work, Ancient Dragon Warriors of the South, topic: present Philippine administration):
- Gather at least 200 records from online sources. Apply the data analytics process.
- Proximity measures: compare users, posts, sources or topics with cosine, Jaccard or Euclidean.
- Topic modeling with LDA, BERTopic, LSA or other. **At least 3 topics.** Evaluate with proper metrics. Discuss the topics.
- Rule-based sentiment classifier (positive, negative, neutral). **90-10 split** (10% unseen for validation), **10-fold cross validation** on the 90%. Evaluate with proper metrics. Show iterative improvement.
- Document sections: cover, team's journey, description, methods, script/output/remarks table, discussion, conclusion, references (with AI use percent), appendix.

### Method steps

1. **Data collection**: 6,699 Reddit comments (r/Philippines, r/newsPH) via the Arctic Shift archive, January to October 2026.
2. **Labeling**: 600 random comments hand-labeled positive, negative, neutral or off topic.
3. **Exploration**: comments per source and month, lengths, label counts, top words before and after stopwords.
4. **Preprocessing**: drop removed/deleted and off-topic comments, clean text (links, gifs, quotes, mentions, lowercase, letters only), drop comments under 5 words and duplicates, custom stopword list of 574 words (English, Filipino/Taglish fillers, plus words in nearly every comment like marcos, bbm). 5,222 comments left, 506 labeled (307 negative, 150 neutral, 49 positive). Sentiment rules read the **original** text because cleaning removes negations, emojis and "!".
5. **Proximity**: TF-IDF vectors. Cosine between sources (average TF-IDF vectors), Jaccard on top 300 words, cosine/Jaccard between topics, Euclidean between sources' topic mixes, and all-pairs cosine to look for copy-paste comments.
6. **Topic modeling**: LDA on word counts, keeping words in at least 5 comments and at most 30% of comments (2,850 words). Tried K = 3 to 8 with 5 seeds each, scored by **NPMI coherence** (higher better) and **perplexity** (lower better). Compared with **LSA** (SVD on TF-IDF).
7. **Rule-based sentiment**: lexicon score summed per comment, then thresholds:
   - Version 1: base lexicon (+1 positive, -1 negative, -2 strong insult).
   - Version 2: + negation (flip the next 3 words after hindi, wala, di, not), intensifiers (sobrang, napaka, very), contrast rule (after pero/but/kaso, earlier part x0.5, later part x1.5).
   - Version 3: + political slang, sarcasm and complaint phrases, emojis, "!" penalty, ALL CAPS boost, and two tuned settings (neutral band width and a lean toward negative), tuned inside each training fold only.
8. **Evaluation**: accuracy, precision, recall, F1 per class, and **macro F1** (plain average of the class F1s, used because classes are imbalanced). Baseline: always guess negative.

### Results to remember

- **K = 3** was best by both coherence (mean NPMI 0.099, falling for K = 4 and 5, negative from 6) and perplexity. Final model coherence 0.111, perplexity 2,275.9.
- Topics: cases and accountability, camp politics (Marcos vs Duterte), corruption and the country.
- **LSA was worse** (coherence -0.237). Negative word weights mixed what a topic is about with what it is not.
- Cosine between sources about 0.69, Jaccard only 0.24 to 0.30. **Cosine is pulled up by a few heavy shared words. Jaccard counts every word equally and sees each source's own long-tail vocabulary.**
- Topics well separated: cosine 0.20 to 0.25, Jaccard 0.026 to 0.053.
- Negative and neutral comments were the most similar classes, which is why a word-based classifier struggles to tell them apart.
- Sentiment macro F1 (10-fold): baseline 0.252 (accuracy 0.607), v1 0.395, v2 0.404, v3 0.491. Unseen 10%: accuracy 0.569, macro F1 0.547.
- **Accuracy misleads on imbalanced data.** The always-negative baseline has the highest accuracy but never finds a neutral or positive comment.
- Biggest gain came from slang, sarcasm phrases and emojis, not grammar rules. Sarcasm is where rules fail.

### Classification metrics (memorize)

```
Accuracy  = (TP + TN) / (all predictions)
Precision = TP / (TP + FP)        of what I called class X, how much really was X
Recall    = TP / (TP + FN)        of the real class X, how much I caught
F1        = 2 x Precision x Recall / (Precision + Recall)
Macro F1  = average of the F1 of each class (each class counts equally)
   TP = true positives, FP = false positives, FN = false negatives, TN = true negatives
```

**k-fold cross validation**: split the data into k parts. Train on k-1, test on the remaining 1. Repeat k times so each part is tested once. Report the mean (and standard deviation). **Stratified** means each fold keeps the same class proportions.

---

# 7. Formula Sheet

```
TEXT ANALYTICS
TF(t,d)        = count of t in d / total terms in d
IDF(t,D)       = log10( N / df(t) )                N = number of docs, df = docs containing t
TF-IDF(t,d)    = TF(t,d) x IDF(t,D)
n-grams in T tokens = T - n + 1

NAIVE BAYES
P(L|features)  = P(features|L) x P(L) / P(features)
P(c|d)         proportional to P(c) x product of P(t_k|c)
P(w_k|c)       = (n_k + 1) / (n + |V|)              n_k = count of w_k in class c, n = all words in class c

VADER
compound       = S / sqrt(S^2 + 15)                 > 0.05 pos, < -0.05 neg, else neutral

LDA
P(W,Z,θ,φ;α,β) = Π_j P(θ_j;α) x Π_i P(φ_i;β) x Π_t P(Z_j,t|θ_j) P(W_j,t|φ_Z_j,t)
p1             = words in doc d with topic k / words in doc d
p2             = assignments of word w to topic k / all words in topic k
new topic      ~ p1 x p2
with priors    ~ (n_dk + α)(n_kw + β) / (n_k + Vβ)
document-topic matrix (docs x K), topic-word matrix (K x vocabulary)
Perplexity     = exp( - sum log p(w_d) / total words )      lower better
NPMI(a,b)      = log[ P(a,b) / (P(a)P(b)) ] / ( -log P(a,b) )  higher better

SPM
support(S)     = number of sequences containing S
standard rule  sup = seqs with X then Y / total seqs
               conf = seqs with X then Y / seqs with X
partially-ordered rule: same, with X and Y unordered, disjoint, non-empty itemsets
GSP join       S1 minus first item == S2 minus last item

PROXIMITY
cos(A,B)       = A.B / (||A|| ||B||)
J(A,B)         = |A ∩ B| / |A ∪ B|
d(A,B)         = sqrt( sum (a_i - b_i)^2 )

TIME SERIES (Activity 2)
additive       y = trend + seasonal + residual
seasonal diff  y(t) - y(t-12)
AIC            = 2k - 2 ln L                         lower better
MAE  = mean |y - ŷ|          RMSE = sqrt(mean (y - ŷ)^2)
MAPE = 100 x mean |(y - ŷ)/y|   R2 = 1 - SS_res / SS_tot

CLASSIFICATION
Precision = TP/(TP+FP)   Recall = TP/(TP+FN)   F1 = 2PR/(P+R)   Accuracy = (TP+TN)/all
```

---

# 8. Likely Exam Questions and Quick Self-Check

## Computation

1. Given a sequence database and min_sup, run GSP: L1, all 2-candidates with supports, joins, pruning, L3, L4. Show which candidates are pruned and why.
2. Is sequence A a subsequence of B? (Watch same-itemset vs separate-itemset.)
3. Compute support of a given sequence, e.g. Sup{(a),(b)} in the 4-row slide table (answer 1).
4. Compute support and confidence of a standard or partially-ordered sequential rule, and decide if it passes given thresholds.
5. Join two given (k-1)-sequences, or say why they cannot be joined.
6. Compute TF, IDF and TF-IDF for 3 short documents after stop word removal, and interpret which words score highest and why some are 0.
7. Build a bag of words / binary occurrence / count table for given sentences. List the bigrams of a sentence.
8. Naive Bayes with add-one smoothing on a small labeled table.
9. VADER compound score from a word-score sum, and classify it.
10. One LDA Gibbs update with p1 x p2.
11. Cosine or Jaccard between two documents.

## Concepts

1. Define text analysis and unstructured data. Give the text formats.
2. List and explain the NLP difficulties with examples (word-level ambiguity, syntactic ambiguity, anaphora, presupposition).
3. List the preprocessing steps in order. Stemming vs lemmatization, with an example where stemming goes wrong.
4. Rule-based vs machine-based vs hybrid classification.
5. Why TF-IDF gives 0 to a word that appears in every document.
6. Dictionary-based vs corpus-based (statistical, semantic) lexicon approaches.
7. VADER vs TextBlob vs SentiWordNet: what each outputs and its steps.
8. LDA's two assumptions. What α, β, θ, φ, z, w, M, N, K mean. Effect of high vs low alpha.
9. What the document-topic and topic-word matrices are and their shapes.
10. Coherence vs perplexity, which direction is better, and how to choose K.
11. pyLDAvis: what λ and the intertopic distance plot show.
12. Sequential rules vs association rules. Why order matters (bed then sheets, word order in text).
13. Apriori property and the join and prune steps.
14. GSP vs SPADE (horizontal vs vertical) vs PrefixSpan (pattern growth, projected databases, no candidates).
15. Standard vs partially-ordered sequential rules.
16. Activity 2: why SARIMAX beat XGBoost, what stationarity is, ADF vs KPSS hypotheses, why XGBoost b overfit, what each metric means.
17. Summative: why macro F1 and not accuracy, why 10-fold, why the 90-10 holdout, why LDA beat LSA, why cosine and Jaccard disagreed.

## Quick self-check (answer without looking)

- {(a,c)} ⊑ {(a),(c)}? (No.)
- {(a),(c)} ⊑ {(a,c),(d)}? (No.)
- Is {(a,b)} the same as {(b,a)}? (Yes.) Is {a, b} the same as {b, a}? (No.)
- Length of <(a),(f,g),(e)>? (4.)
- Can {a,(b,f)} join with {b,f,c}? (No, (b,f) is one event, b,f are two.)
- What does a high alpha do to documents? (More topics per document.)
- Lower or higher perplexity is better? (Lower.) Coherence? (Higher.)
- IDF of a word in 2 of 3 documents? (log 1.5 = 0.176.)
- VADER neutral band? (-0.05 to 0.05.)
- ADF null hypothesis? (Not stationary.) KPSS null? (Stationary.)
- Who introduced SPM, and when? (Agrawal and Srikant, 1994.)
- GSP data layout? (Horizontal.) SPADE? (Vertical.)
