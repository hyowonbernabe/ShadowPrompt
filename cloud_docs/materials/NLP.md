# CSE 30 (NLP) Midterm Exam Reference Notes

CSE 30, Special Topics 1 (Natural Language Processing and Artificial Neural Networks), Saint Louis University, SAMCIS, BSCS 4. Facilitator: Dalos D. Miguel. Exam: November 6, 2026, 1:00 PM.

Exam coverage, as announced (and repeated on the first slide of ANN Part 4, "Discussed Theories for the Midterm Exam"):

1. Naive Bayes classification algorithm, applied to language classification
2. The McCulloch & Pitts model
3. The Perceptron model and its training algorithm
4. The Backpropagation network (training algorithm)
5. General information related to NLP

Sources: ANN Parts 1 to 4 (midterm slides), ANN Part 5 (labeled final, used only where it overlaps backprop and ANN applications, and marked as Part 5), the NaiveBayes_LanguageData.xlsx spreadsheet, the NLP course overview slides, the syllabus, and last term's prelim reviewer. Anything I filled in from general knowledge is marked **[general knowledge]**.

---

## 1. General NLP (condensed from the prelim notes)

### 1.1 What NLP is

- **NLP** is the software or hardware parts of a computer system that analyze and produce (synthesize) spoken or written language. That is the course slides' definition.
- Konasani & Kadre (2021): NLP uses Artificial Intelligence to recognize human speech or conversation in any language.
- Working standard: what NLP produces should be close to what a human would produce doing the same interpretation.

### 1.2 The two core subsets (Khurana et al., 2018)

1. **Natural Language Understanding (NLU)**: understanding a phrase, sentence, or paragraph. Falls under linguistics and splits into phonology, morphology, syntax, semantics, and pragmatics (discourse is listed alongside).
2. **Natural Language Generation (NLG)**: producing a phrase, sentence, or paragraph as text.

A current approach built on top of this is **Retrieval Augmented Generation (RAG)**, where a model looks up relevant documents first and then generates an answer from them.

### 1.3 NLP concern examples from the slides

- Morphological analysis (lemmatization, stemming): "pag-iibigan" breaks into "pag-", "i-", "ibig", "-an".
- Machine translation: "Umali Kayo." becomes "You are welcome here." or "You come here."
- Sentiment analysis: "Food is tasty and affordable at PCC" is Positive ("You should visit the restaurant.").
- Entity recognition: pulling "14 February 2023" out of a longer sentence as a date.

### 1.4 Core NLP tasks (from the slides)

- **Automatic Summarization**: makes a readable summary of a body of text.
- **Co-Reference Resolution**: figures out which words refer to the same thing.
- **Discourse Analysis**: finds the structure of connected text.
- **Machine Translation**: translates text from one human language to another.
- **Morphological Segmentation**: splits a word into morphemes and labels each one.
- **Named Entity Recognition (NER)**: finds which items in text are proper names.
- **Optical Character Recognition (OCR)**: reads printed text from an image.
- **Part of Speech Tagging**: labels each word with its part of speech.

These tasks overlap. Some act as subtasks inside bigger ones.

### 1.5 NLP spectrum (as pictured)

Corpus Building -> Morphological Analysis -> Segmentation -> Tokenization -> Stemming -> POS Tagging -> Language Translation -> Sentiment Analysis -> Topic Modeling -> (more).

### 1.6 Terminologies (linguistic levels)

Phonology, Morphology, Lexical Level, Syntactic Level, Semantic Level, Discourse, Pragmatic Level.

### 1.7 Applications of NLP

Machine Translation, Text Categorization, Spam Filtering, Information Extraction, Summarization, Dialog Systems, Medicine (Specialist Systems).

### 1.8 Areas of study under NLP (as organized for this course)

- Sentiment Analysis
- Topic Modeling
- Morphological Analysis (electronic lexicon, morphology, stemming)
- Part of Speech Tagging (tagset, rule-based and probabilistic approaches)
- Machine Translation (unidirectional, bidirectional)

### 1.9 Course goals and term structure

- Goals: build (online) resources for natural languages used in the Cordillera Administrative Region (CAR) and nearby regions, build domain-based electronic lexicons, and create NLP-based eGovernance tools for institutions or social units.
- Prelim: domain-based collection and preprocessing of resources (example: Pangasinan Word-POS-English Translation).
- Midterm: a newly created resource for a selected language (example: Pangasinan lexicon for agriculture, Kapampangan lexicon for culinary arts).
- Final: a journal paper presenting and applying an NLP algorithm (classification, correlation, translation, sentiment analysis).
- The syllabus says the course covers ANN fundamentals (McCulloch and Pitts, Perceptron, Backpropagation) because current NLP tools build on them. Syllabus outcome 11 ties the Perceptron and the Backpropagation network to word classification in NLP research.

### 1.10 Quick refreshers on prelim topics

- **POS tagging**: assigning the correct part of speech to every word. Example: "Siya ay inaantok." gets PRS LM JJD PMP. A **tagset** is the list of tags allowed (Rabo 2004 Tagalog tagset: 59 tags, Buban's revised tagset: 65 tags, plus Penn Treebank, CLAWS, Universal). Approaches are rule-based (word database plus rules), stochastic (probabilities, n-grams, Hidden Markov Model, Viterbi), and neural. Stochastic tagging picks the tag sequence T for words W that maximizes `P(T|W) = P(T) * P(W|T) / P(W)`, the same Bayes idea used in Naive Bayes below.
- **Bag of Words (BoW)**: turns text into a vector of word counts over a fixed vocabulary. It keeps which words appear and how often, and throws away word order.
- **Morphology**: the study of word structure. A **morpheme** is the smallest meaningful piece (root, prefix, suffix). **Inflection** makes a word of the same part of speech (talk -> talked). **Derivation** changes the meaning or class (malice -> malicious). Other phenomena: incorporation, clitics, compounding, infixation, prefixation, suffixation, circumfixation, reduplication.
- **Transformer / BERT** (prelim readings): the Transformer replaces recurrence with attention, `Attention(Q, K, V) = softmax(Q K^T / sqrt(d_k)) V`. BERT is a pretrained Transformer encoder used for tasks like topic modeling (BERTopic) and sentiment work.

### 1.11 Evaluation metrics (EvaluationMetrics sheet of the Naive Bayes workbook)

For a category Cn:

| Term | Meaning (as in the sheet) | Shorthand in sheet |
|---|---|---|
| True Positive (TP) | members of Cn that were classified as Cn | Cn/Cn |
| False Positive (FP) | nonmembers of Cn that were classified as Cn | Not Cn/Cn |
| False Negative (FN) | members of Cn that were not classified as Cn | Cn/Not Cn |
| True Negative (TN) | nonmembers of Cn that were not classified as Cn | |

- `Precision = TP / (TP + FP)` (of everything I labeled Cn, how much really was Cn)
- `Recall = TP / (TP + FN)` (of everything that really was Cn, how much I caught)
- `Accuracy = (all TPs across categories) / Total` (share of all test items classified correctly)

---

## 2. Naive Bayes for Language Classification

### 2.1 The idea

**Naive Bayes** is a classifier that picks the class with the highest probability given the evidence, using Bayes' rule. It is "naive" because it assumes every feature (here, every word) is independent of the others once you know the class.

Bayes' rule for language L and text W = w1 w2 ... wn:

```
P(L | W) = P(L) * P(W | L) / P(W)
```

- `P(L | W)`: probability the text is in language L, given its words (what I want).
- `P(L)`: the **prior**, how likely language L is before looking at the text.
- `P(W | L)`: the **likelihood**, how likely these words are if the text is in L.
- `P(W)`: probability of the text itself. It is the same for every language, so it is dropped when comparing.

With the naive independence assumption:

```
P(W | L) = P(w1 | L) * P(w2 | L) * ... * P(wn | L)

score(L) = P(L) * P(w1 | L) * P(w2 | L) * ... * P(wn | L)

predicted language = the L with the highest score
```

### 2.2 How the instructor's spreadsheet sets it up

The workbook NaiveBayes_LanguageData.xlsx has four sheets: DistributionOfWords, MostFrequentWordsFromCorpora, EvaluationMetrics, Exercise.

- **Classes**: 8 Philippine languages: Bikol, Cebuano, Hiligaynon, Ilocano, Kapampangan, Pangasinan (Pangasinense), Tagalog, Waray.
- **Features**: whole words (not letters, not n-grams). For each language the sheet keeps the **20 most frequent words** from that language's corpus, plus one extra row called **Others** with frequency 0. Others stands for every word not in the top 20.
- **Laplace (add-one) smoothing**: every count gets +1 (column "freq+1"). Smoothing means adding a small count so that a word never seen in a language does not get probability 0 and wipe out the whole product. So each top-20 word has `freq + 1`, and Others has `0 + 1 = 1`.
- **Language total**: `TOTAL FREQ.(L) = sum of the 21 "freq+1" values` (20 words plus Others).
- **Grand total**: `Total Freq (All Languages) = sum of the 8 language totals = 784,438`.

Formulas, exactly as the sheet builds them:

```
Prior:       P(L)     = TOTAL FREQ.(L) / Total Freq (All Languages)
Likelihood:  P(w | L) = (freq of w in L + 1) / TOTAL FREQ.(L)
Unseen word: P(w | L) = (0 + 1) / TOTAL FREQ.(L)      (uses the Others row)
```

Note: the prior here is based on word counts, not on how many documents each language has. The sheet computes the priors; the per-sentence classification is left to us (Exercise sheet), so the worked examples below apply the sheet's own numbers.

### 2.3 Priors (from the sheet)

| Language | TOTAL FREQ. (with +1) | P(L) |
|---|---|---|
| Bikol | 96,904 | 0.123533 |
| Cebuano | 104,478 | 0.133188 |
| Hiligaynon | 114,979 | 0.146575 |
| Ilocano | 101,190 | 0.128997 |
| Kapampangan | 75,483 | 0.096226 |
| Pangasinan | 76,658 | 0.097723 |
| Tagalog | 109,854 | 0.140042 |
| Waray | 104,892 | 0.133716 |
| **All** | **784,438** | **1.000000** |

### 2.4 The word lists (raw freq, before the +1)

- **Bikol**: alagad 1571, an 14554, asin 4149, dai 2488, huli 1600, ini 1396, jesus 1425, kan 11546, kun 1452, man 1625, mga 6992, na 15404, ni 2471, nin 5903, niya 1445, sa 15000, si 2859, sinda 1462, siya 1841, ta 1700
- **Cebuano**: ang 14068, apan 1539, dili 1829, iyang 2689, jesus 1340, ka 2890, kay 1341, ko 1783, mga 7117, na 2212, nga 10921, ni 4106, niya 1540, sa 30849, si 3584, sila 1392, siya 2552, ug 9803, usa 1413, wala 1489
- **Hiligaynon**: ang 14680, gid 1979, ila 1818, indi 2081, iya 5423, ka 3531, kag 6933, kay 2134, kon 1797, man 2038, mga 7068, na 2990, nga 17717, ni 3804, niya 2786, sa 17343, sang 12973, si 3655, sia 1864, wala 2344
- **Ilocano**: a 20251, adda 1722, dagiti 3976, daytoy 1123, idi 1585, isu 1071, iti 12592, kadagiti 1309, ken 5055, ket 5088, maysa 1248, met 2112, nga 7740, ngem 1512, ni 6866, no 2368, pay 1614, saan 1212, ta 1929, ti 20796
- **Kapampangan**: a 8170, at 10486, ding 4719, e 2064, genesis 1532, i 1977, ing 8352, karing 1807, keang 1753, kekang 1609, king 9538, ku 1941, la 1838, mu 1828, n 1576, na 4774, nang 2405, ng 1512, ning 4530, ya 3051
- **Pangasinan**: a 7253, ag 2365, da 2141, diad 1498, ed 8241, et 1979, ko 1722, la 1668, mo 1554, na 8295, nen 2549, no 2872, saray 3014, say 3029, si 1925, so 6314, ta 2614, tan 6277, to 4556, ya 6771
- **Tagalog**: ang 17915, at 9788, ay 3215, hindi 2278, isang 1829, ito 2623, kanyang 2303, ko 2029, kung 1633, may 1963, mga 8547, na 11161, nang 1350, ng 15266, ni 1989, niya 1871, pa 1467, sa 18684, si 2007, siya 1915
- **Waray**: af 4788, an 12641, dbch 2328, f 2619, ha 8478, han 8626, hich 2328, hin 2784, iya 2758, ka 3203, kay 2075, la 1995, loch 2328, mga 7777, na 3874, nga 19655, ngan 5638, sa 5893, san 3139, si 1944

Every language also has Others = 0 (becomes 1 after smoothing). Some Waray entries (af, dbch, f, hich, loch) look like noise from the source corpus, but they are in the sheet as given. Most of the top words are function words (ang, sa, nga, mga, na), which is why they work as language markers.

### 2.5 Fully worked example: "ang lahat ay nakapangyayari" (Exercise item 3)

**Step 1. Split into words (tokens).** ang, lahat, ay, nakapangyayari.

**Step 2. Look up each word in each language.** If the word is in the top 20, use `freq + 1`. If not, use Others = 1.

| Language | ang | lahat | ay | nakapangyayari |
|---|---|---|---|---|
| Bikol | 1 | 1 | 1 | 1 |
| Cebuano | 14069 | 1 | 1 | 1 |
| Hiligaynon | 14681 | 1 | 1 | 1 |
| Ilocano | 1 | 1 | 1 | 1 |
| Kapampangan | 1 | 1 | 1 | 1 |
| Pangasinan | 1 | 1 | 1 | 1 |
| Tagalog | 17916 | 1 | 3216 | 1 |
| Waray | 1 | 1 | 1 | 1 |

**Step 3. Turn counts into likelihoods** by dividing by that language's total.

Tagalog (total 109,854):

```
P(ang | Tagalog)            = 17916 / 109854 = 0.163089
P(lahat | Tagalog)          =     1 / 109854 = 0.0000091030
P(ay | Tagalog)             =  3216 / 109854 = 0.029275
P(nakapangyayari | Tagalog) =     1 / 109854 = 0.0000091030
```

Cebuano (total 104,478):

```
P(ang | Cebuano)   = 14069 / 104478 = 0.134660
each other word    =     1 / 104478 = 0.0000095714
```

Hiligaynon (total 114,979):

```
P(ang | Hiligaynon) = 14681 / 114979 = 0.127684
each other word     =     1 / 114979 = 0.0000086972
```

Languages where all four words are unseen: every likelihood is `1 / total`.

**Step 4. Multiply prior by all likelihoods.**

```
score(Tagalog)    = 0.140042 * 0.163089 * 0.0000091030 * 0.029275 * 0.0000091030
                  = 5.5405 x 10^-14
score(Cebuano)    = 0.133188 * 0.134660 * (0.0000095714)^3 = 1.5726 x 10^-17
score(Hiligaynon) = 0.146575 * 0.127684 * (0.0000086972)^3 = 1.2312 x 10^-17
score(Kapampangan)= 0.096226 * (1/75483)^4  = 2.9641 x 10^-21
score(Pangasinan) = 0.097723 * (1/76658)^4  = 2.8299 x 10^-21
score(Bikol)      = 0.123533 * (1/96904)^4  = 1.4009 x 10^-21
score(Ilocano)    = 0.128997 * (1/101190)^4 = 1.2304 x 10^-21
score(Waray)      = 0.133716 * (1/104892)^4 = 1.1046 x 10^-21
```

**Step 5. Pick the largest.** Tagalog wins (5.54 x 10^-14), about 3,500 times bigger than the runner-up, Cebuano. It wins because it is the only language that knows both "ang" and "ay".

**Optional: log probabilities [general knowledge].** The sheet multiplies directly. With many words the numbers get tiny, so in code you add logs instead: `log score(L) = log P(L) + sum of log P(wi | L)`. The winner does not change. Here, log10 scores are Tagalog -13.26, Cebuano -16.80, Hiligaynon -16.91, and about -20.5 to -21.0 for the rest.

### 2.6 The other exercise items (same method)

| Item | Text | Key words found | Winner | Runner-up |
|---|---|---|---|---|
| 1 | pagkatapos naglabar ako | none (all three unseen everywhere) | Kapampangan (2.237 x 10^-16) | Pangasinan (2.169 x 10^-16) |
| 2 | gikuhaan sa prutas | "sa" (Cebuano 30850, Tagalog 18685, Hiligaynon 17344, Bikol 15001, Waray 5894) | Cebuano (3.603 x 10^-12) | Bikol (2.036 x 10^-12) |
| 3 | ang lahat ay nakapangyayari | "ang", "ay" | Tagalog (5.541 x 10^-14) | Cebuano (1.573 x 10^-17) |
| 4 | pagkamaopay hito nga sumal | "nga" (Waray 19656, Hiligaynon 17718, Cebuano 10922, Ilocano 7741) | Waray (2.171 x 10^-17) | Hiligaynon (1.486 x 10^-17) |

Item 2 detail: `Cebuano = 0.133188 * (1/104478) * (30850/104478) * (1/104478) = 3.603 x 10^-12`. Cebuano wins because "sa" is by far its most common word (30,849).

Item 4 detail: `Waray = 0.133716 * (1/104892)^3 * (19656/104892) = 2.171 x 10^-17`, against `Hiligaynon = 0.146575 * (1/114979)^3 * (17718/114979) = 1.486 x 10^-17`. Close, but Waray wins on the "nga" count.

Item 1 is a trap worth knowing. No word is known, so the score is `P(L) * (1/total)^3`. Smaller totals give bigger `1/total`, and cubing that outweighs the smaller prior. So the language with the smallest corpus (Kapampangan) wins. The result says more about smoothing than about the text. "ako" is a common Tagalog word, but it is not in Tagalog's top 20 in this sheet.

### 2.7 Steps to remember

1. Tokenize the text into words.
2. For each language, get `freq + 1` for each word (Others = 1 if missing).
3. Divide by the language total to get `P(w | L)`.
4. Multiply the prior `P(L)` by all the `P(w | L)`.
5. The highest product is the answer.

---

## 3. ANN Background (Parts 1 and 2)

### 3.1 Why ANNs (Part 1)

- Pattern recognition (shapes, handwritten digits) is easy for people and hard to program on a conventional computer.
- A conventional computer cannot learn from experience. It only runs explicit instructions written by a programmer.
- A computer is built from simple processing elements (logic gates). The brain is built from biological counterparts (neurons).
- Computer parts switch several orders of magnitude faster than neurons, but the brain's architecture is radically different.
- A computer is basically a **sequential** processor. The brain uses **massive parallelism and interconnectivity**, and that is where its abilities come from.
- **Artificial Neural Networks (ANN)** borrow features from the brain's physiology as the basis of processing models. They mimic how the brain processes information. The field is also called **Parallel Distributed Processing**.
- Underlying principles of ANN: **Adaptation** and **Learning**.
- ANN is interdisciplinary: **Neuroscience, Mathematics, Computer Science**.
- Example network in the slides: handwritten digit recognition with 80 input units (input layer), a hidden layer, and 10 output units (digits 0 to 9), joined by weighted connections.

**Typical input characteristics (training mode):**

- Able to "learn" by example.
- Does not need a well-defined algorithm for turning input into output.
- Adapts itself when shown representative examples, so it reproduces the desired output for those inputs.
- Finds its own solution, given only examples of the desired behavior.

**Typical output characteristics (production mode):**

- Uses "learned" information to respond to inputs.
- Can respond to, and generalize from, "noisy" input.
- **Fault tolerant**: minor "damage" to the network may have little effect on performance.

Main reference: Freeman, J. & Skapura, D. (1991). *Neural Networks: Algorithms, Applications and Programming Techniques.* Addison-Wesley.

### 3.2 Neurophysiology (Parts 1 and 2)

Parts of a neuron in the slides: dendrites, cell body, nucleus, axon hillock, axon, myelin sheath, nodes of Ranvier, synapse. Synaptic junction parts: presynaptic membrane, synaptic vesicle, neurotransmitter release, synaptic cleft, postsynaptic membrane.

**Neuron basic characteristics:**

- The cell membrane is selectively permeable to certain ions. At rest (quiescent state) it keeps a potential difference of **70 to 100 millivolts** across the membrane.
- The axon connects to other neurons (several to thousands) through synapses on the **dendritic tree**, the branch-like parts off the cell body. This is the neuron's **input**.
- Potential changes at the synapses are **summed in the cell body**. This is the **processing**.
- Based on that sum, the neuron either stays quiet or gets excited and **fires**, sending an electrical spike down the axon. This is the **output**.
- The basic architecture may have a genetic blueprint but varies from brain to brain, probably because of different "trainings".

**Synaptic junction activity:**

- Synapses act like "chemical contacts". The electrical pulse in the axon makes the synapse release **neurotransmitters** (stored in vesicles near the presynaptic membrane).
- Depending on the neurotransmitter, the cell body's potential either **rises (excitatory synapse)** or **drops (inhibitory synapse)**.
- These rises and drops are summed. When the sum exceeds a **threshold**, the neuron fires and an **action potential** travels down the axon.
- After firing, the neuron returns to its resting potential and cannot fire again for about **1 millisecond**. This **refractory period** limits firing to about **1000 pulses per second**.
- Connections can have **feedback paths**, so a neuron's activity can come back to itself through the neurons it affects.

---

## 4. The McCulloch & Pitts Model (Part 2, deficiencies in Part 3)

### 4.1 Assumptions (the five rules on the slide)

1. The activity of a neuron is an **all-or-none** process. Neurons are binary (fire or not fire).
2. A fixed number of synapses must be excited within a period of **latent addition** for a neuron to fire. In the slides: **at least 2 excitatory inputs cause a neuron to fire**.
3. The only significant delay in the nervous system is **synaptic delay** (one time step per neuron).
4. The activity of any **inhibitory synapse absolutely prevents** the neuron from firing at that time.
5. The structure of the interconnection network **does not change with time**.

### 4.2 Structure and conventions

- Neurons are numbered circles. A line from neuron A ends at neuron B in a synapse.
- **Filled dot** = excitatory synapse. **Hollow (open) dot** = inhibitory synapse.
- A line that splits into **two filled dots** gives **two** excitatory synapses, so that neuron alone is enough to fire the target (2 meets the threshold).
- A line with **one filled dot** gives only one synapse, so it needs a partner to reach 2.

Notation:

- `Ni(t)`: neuron i fires at time t.
- `¬Ni(t)` (written with a bar over it on the slide): neuron i did not fire at time t.
- `v`: disjunction (OR).
- `&`: conjunction (AND).
- `:=` means "is defined as". `t-1` means one step earlier (one synaptic delay).

As a threshold model (my reading of the rules): each excitatory synapse adds 1, the **threshold is 2**, and any active inhibitory synapse blocks firing no matter what the sum is.

### 4.3 The slide examples

| Example | Wiring | Expression |
|---|---|---|
| 1. Precession | 1 -> 2 with a double synapse | `N2(t) := N1(t-1)` |
| 2. Disjunction (OR) | 1 -> 3 double, 2 -> 3 double | `N3(t) := N1(t-1) v N2(t-1)` |
| 3. Conjunction (AND) | 1 -> 3 single, 2 -> 3 single | `N3(t) := N1(t-1) & N2(t-1)` |
| 4. Conjoined negation | 1 -> 3 double, 2 -> 3 inhibitory | `N3(t) := N1(t-1) & ¬N2(t-1)` |

Why each one works:

- **Precession**: neuron 1 alone gives 2 synapses, so neuron 2 copies neuron 1 one step later.
- **OR**: either input alone gives 2, so either one fires neuron 3.
- **AND**: each input gives only 1. Both are needed to reach 2.
- **Conjoined negation (AND NOT)**: neuron 1 gives 2, but if neuron 2 fires, its inhibitory synapse blocks neuron 3.

Truth tables (inputs at t-1, output at t):

| N1 | N2 | OR | AND | N1 & ¬N2 |
|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 |
| 0 | 1 | 1 | 0 | 0 |
| 1 | 0 | 1 | 0 | 1 |
| 1 | 1 | 1 | 1 | 0 |

### 4.4 Example 5 (the slide's question): N3(t) := ? N4(t) := ?

Wiring read from the slide: neuron 1 -> 3 (double). Neuron 2 -> b (double), 2 -> a (inhibitory), 2 -> 4 (single). b -> 4 (single) and b -> a (double). a -> 3 (double).

Working it out:

```
Nb(t) := N2(t-1)
Na(t) := Nb(t-1) & ¬N2(t-1)          = N2(t-2) & ¬N2(t-1)
N3(t) := N1(t-1) v Na(t-1)           = N1(t-1) v [ N2(t-3) & ¬N2(t-2) ]
N4(t) := N2(t-1) & Nb(t-1)           = N2(t-1) & N2(t-2)
```

**Answer:** `N3(t) := N1(t-1) v [N2(t-3) & ¬N2(t-2)]` and `N4(t) := N2(t-1) & N2(t-2)`.

[general knowledge] This is McCulloch and Pitts' classic heat and cold illusion. Neuron 1 is a heat receptor, neuron 2 a cold receptor, neuron 3 "feel hot", neuron 4 "feel cold". A brief touch of cold (one step, then gone) makes you feel heat. A cold touch lasting two steps makes you feel cold.

### 4.5 The exercise: build N3(t) := N2(t-2) & N1(t-3)

Neuron 3 needs neuron 1's signal delayed 3 steps and neuron 2's signal delayed 2 steps, then AND them.

- Chain for neuron 1: 1 -> a (double), a -> b (double), b -> 3 (single). So `Nb(t-1) = N1(t-3)`.
- Chain for neuron 2: 2 -> c (double), c -> 3 (single). So `Nc(t-1) = N2(t-2)`.
- Neuron 3 gets two single synapses (from b and c), so it is an AND: `N3(t) := Nb(t-1) & Nc(t-1) = N1(t-3) & N2(t-2)`.

Rule of thumb: each extra neuron in a chain adds one step of delay. A double synapse passes a signal through alone. Single synapses into the same neuron make an AND.

### 4.6 NOT and XOR [general knowledge, the slides only show conjoined negation]

- **NOT** by itself (`N2(t) := ¬N1(t-1)`) needs a neuron that is always firing (a constant source) with a double synapse into neuron 2, plus an inhibitory synapse from neuron 1. Then neuron 2 fires unless neuron 1 fires. That is just conjoined negation with the first input stuck on.
- **XOR** cannot be done by **one** M&P neuron, because one neuron only does OR, AND, or AND NOT style rules on its inputs. It can be done with a **second layer**: neuron a computes `N1 & ¬N2`, neuron b computes `N2 & ¬N1`, and neuron 3 ORs them: `N3(t) := [N1(t-2) & ¬N2(t-2)] v [N2(t-2) & ¬N1(t-2)]`. This lines up with the perceptron story in Section 6.6: one layer cannot do XOR, an added middle layer can.

### 4.7 Deficiencies of the M&P model (Part 3)

- How are the needed connections between neurons formed, especially through **learning**? (The weights and wiring are fixed by hand.)
- How can these networks get the **fault tolerance** of real neurons?

---

## 5. Hebbian Learning (Part 3, bridge to the Perceptron)

Hebb's postulate (*The Organization of Behavior*, 1949): when an axon of cell A is near enough to excite cell B and repeatedly takes part in firing it, some growth or metabolic change happens in one or both cells so that A's efficiency at firing B goes up.

The slides' conditioning example:

1. Sight input C is enough to fire B (salivation signal).
2. Sound input A alone is not enough to fire B.
3. A is active each time C fires B.
4. A metabolic change makes A alone enough to fire B.

How it is explained:

- Hebb: the area of the synaptic junction increases.
- More recent theories: the presynaptic cell releases neurotransmitter at a higher rate.
- In ANNs: this is done by adjusting the **strength of the synaptic weights**.

Part 5 adds (final slides): **Hebb's Rule** was the first learning rule. If a neuron receives input from another neuron and both are highly active (same sign), the weight between them should be strengthened.

---

## 6. The Perceptron (Parts 3 and 4)

### 6.1 The general processing element (PE)

A **processing element** is one artificial neuron. It takes weighted inputs, adds them, and passes the sum through an output function.

```
Net_i = sum over j of ( W_ij * X_j )
X_i   = f( Net_i )
```

- `W_ij`: weight of the connection **from unit j to unit i**. A real number. Every input connection has one.
- `X_j`: output of unit j (an input to unit i).
- `Net_i`: the **stimulation signal** (net input) to unit i.
- `f`: the output (activation) function.
- `X_i`: output of unit i. It is a single output that can go to many other PEs.

### 6.2 Output functions

**Binary threshold function** (θ = threshold value):

```
X_i = f(Net_i) = 1   if Net_i >= θ
X_i = f(Net_i) = 0   if Net_i <  θ
```

**Equivalent form** (move θ to the left side):

```
X_i = 1   if Net_i - θ >= 0
X_i = 0   if Net_i - θ <  0
```

**Sigmoid function** (other output function on the slide):

```
f(x) = 1 / (1 + e^-x)
f(x) = 1 / (1 + e^-2x)      (steeper version)
```

The Part 3 slide prints these as `1/(1+e^x)` and `1/(1+e^2x)`. The minus sign in the exponent is lost in the slide text. Part 4's BPN problem states it correctly as `f(x) = 1/(1+e^-x)`. The sigmoid squashes any number into the range 0 to 1.

### 6.3 Network with no threshold (bias handling)

The slide: fix an extra input at **1** with a weight equal to the threshold value. That way the threshold becomes just another weight that can be stored and trained like the others. [general knowledge] To make the math line up with `Net - θ >= 0`, the constant input contributes `-θ` (weight θ on an input of -1, or weight -θ on an input of 1), and the unit then fires when the total is `>= 0`. This weight is what is usually called the **bias**.

### 6.4 Perceptron learning procedure (training algorithm)

For each iteration (cycle) t:

1. Present an input vector.
2. Present the desired output (`Yc`).
3. Compute the perceptron's actual output (`Y`) using the output function.
4. If `Y ≠ Yc`, adjust the weights:

```
W_ij(t+1) = W_ij(t) + r * ( Yc - Y ) * X_i
```

5. Repeat from step 1 until the weights stay constant for all possible inputs for one whole **epoch**.

Symbols:

- `W_ij(t)`: weight before the update. `W_ij(t+1)`: weight after.
- `r`: the **learning rate**, a small number that sets how big each adjustment is.
- `Yc`: correct (desired, target) output. `Y`: actual output.
- `X_i`: the input applied on that connection (the slide says "Xi is the input applied for the connection").
- **Epoch**: one pass through every training pattern once.

What the rule does:

- `Yc - Y = 0`: output was right, no change.
- `Yc - Y = +1`: output was 0 but should be 1, so raise the weights on active inputs.
- `Yc - Y = -1`: output was 1 but should be 0, so lower the weights on active inputs.
- An input of 0 means that weight does not change, since it did not contribute.

### 6.5 Worked training trace: the OR perceptron (Part 4 "Training Illustration")

Setup from the slides:

- Two inputs x1, x2, one output unit. Target is `Yc = x1 OR x2`.
- Initial ("wrong") weights: `W11 = 0.2`, `W12 = 0.2`.
- Threshold `θ = 0.5` (kept fixed, only weights are trained).
- Learning rate `r = 0.05`.
- Output: `Y = 1 if W11*x1 + W12*x2 >= 0.5, else 0`.
- Pattern order each epoch: (1,1), (1,0), (0,1), (0,0).

Each row uses the weights shown on that row. The update result appears on the next row.

| Time | x1 | x2 | W11 | W12 | Net | Y | Yc | Yc - Y |
|---|---|---|---|---|---|---|---|---|
| 1 | 1 | 1 | 0.20 | 0.20 | 0.40 | 0 | 1 | 1 |
| 2 | 1 | 0 | 0.25 | 0.25 | 0.25 | 0 | 1 | 1 |
| 3 | 0 | 1 | 0.30 | 0.25 | 0.25 | 0 | 1 | 1 |
| 4 | 0 | 0 | 0.30 | 0.30 | 0.00 | 0 | 0 | 0 |
| 5 | 1 | 1 | 0.30 | 0.30 | 0.60 | 1 | 1 | 0 |
| 6 | 1 | 0 | 0.30 | 0.30 | 0.30 | 0 | 1 | 1 |
| 7 | 0 | 1 | 0.35 | 0.30 | 0.30 | 0 | 1 | 1 |
| 8 | 0 | 0 | 0.35 | 0.35 | 0.00 | 0 | 0 | 0 |
| 9 | 1 | 1 | 0.35 | 0.35 | 0.70 | 1 | 1 | 0 |
| 10 | 1 | 0 | 0.35 | 0.35 | 0.35 | 0 | 1 | 1 |
| 11 | 0 | 1 | 0.40 | 0.35 | 0.35 | 0 | 1 | 1 |
| 12 | 0 | 0 | 0.40 | 0.40 | 0.00 | 0 | 0 | 0 |
| 13 | 1 | 1 | 0.40 | 0.40 | 0.80 | 1 | 1 | 0 |
| 14 | 1 | 0 | 0.40 | 0.40 | 0.40 | 0 | 1 | 1 |
| 15 | 0 | 1 | 0.45 | 0.40 | 0.40 | 0 | 1 | 1 |
| 16 | 0 | 0 | 0.45 | 0.45 | 0.00 | 0 | 0 | 0 |
| 17 | 1 | 1 | 0.45 | 0.45 | 0.90 | 1 | 1 | 0 |
| 18 | 1 | 0 | 0.45 | 0.45 | 0.45 | 0 | 1 | 1 |
| 19 | 0 | 1 | 0.50 | 0.45 | 0.45 | 0 | 1 | 1 |
| 20 | 0 | 0 | 0.50 | 0.50 | 0.00 | 0 | 0 | 0 |
| 21 | 1 | 1 | 0.50 | 0.50 | 1.00 | 1 | 1 | 0 |
| 22 | 1 | 0 | 0.50 | 0.50 | 0.50 | 1 | 1 | 0 |
| 23 | 0 | 1 | 0.50 | 0.50 | 0.50 | 1 | 1 | 0 |
| 24 | 0 | 0 | 0.50 | 0.50 | 0.00 | 0 | 0 | 0 |

Sample updates:

```
Time 1:  W11 = 0.20 + 0.05*(1-0)*1 = 0.25    W12 = 0.20 + 0.05*(1-0)*1 = 0.25
Time 2:  W11 = 0.25 + 0.05*(1-0)*1 = 0.30    W12 = 0.25 + 0.05*(1-0)*0 = 0.25
Time 3:  W11 = 0.30 + 0.05*(1-0)*0 = 0.30    W12 = 0.25 + 0.05*(1-0)*1 = 0.30
```

Epoch summary:

| Epoch | Times | Weights at start (W11, W12) | Errors in epoch |
|---|---|---|---|
| 1 | 1 to 4 | 0.20, 0.20 | 3 |
| 2 | 5 to 8 | 0.30, 0.30 | 2 |
| 3 | 9 to 12 | 0.35, 0.35 | 2 |
| 4 | 13 to 16 | 0.40, 0.40 | 2 |
| 5 | 17 to 20 | 0.45, 0.45 | 2 |
| 6 | 21 to 24 | 0.50, 0.50 | 0 (highlighted on the slide, training stops) |

**Final model** (Part 4 "OR perceptron with correct weights after training"): `W11 = 0.5`, `W12 = 0.5`, `θ = 0.5`. Check: (0,0) -> 0. (0,1) and (1,0) -> 0.5 >= 0.5 -> 1. (1,1) -> 1.0 -> 1. That is OR.

Watch the `>=`. At time 22 the net is exactly 0.5, and since the rule is `Net >= θ`, the output is 1. If you use `>` you get the wrong answer.

### 6.6 AND perceptron (same method) [my own trace, the slide only shows the empty diagram]

Same start: `W11 = W12 = 0.2`, `θ = 0.5`, `r = 0.05`, target `x1 AND x2`.

| Time | x1 | x2 | W11 | W12 | Net | Y | Yc | Yc - Y |
|---|---|---|---|---|---|---|---|---|
| 1 | 1 | 1 | 0.20 | 0.20 | 0.40 | 0 | 1 | 1 |
| 2 | 1 | 0 | 0.25 | 0.25 | 0.25 | 0 | 0 | 0 |
| 3 | 0 | 1 | 0.25 | 0.25 | 0.25 | 0 | 0 | 0 |
| 4 | 0 | 0 | 0.25 | 0.25 | 0.00 | 0 | 0 | 0 |
| 5 | 1 | 1 | 0.25 | 0.25 | 0.50 | 1 | 1 | 0 |
| 6 | 1 | 0 | 0.25 | 0.25 | 0.25 | 0 | 0 | 0 |
| 7 | 0 | 1 | 0.25 | 0.25 | 0.25 | 0 | 0 | 0 |
| 8 | 0 | 0 | 0.25 | 0.25 | 0.00 | 0 | 0 | 0 |

Epoch 2 has no errors, so the final AND model is `W11 = W12 = 0.25`, `θ = 0.5`. Any weights where each one alone is below θ but the two together reach θ will do AND.

The Part 3 pair exercise asked for the same three things for OR: the initial model (random weights and parameters), the training data and results in a table, and the final model (final weights).

### 6.7 Convergence and linear separability

- **Convergence is guaranteed when the input space is linearly separable.** Convergence means training reaches weights that stop changing because every pattern is right.
- **Linearly separable** (slide definition): plot every input as a point on a plane. If a line (or plane, in more dimensions) can be drawn so that all points of one category are on one side, the input space is linearly separable.
- **AND**: (1,1) is the only 1. A line like `x1 + x2 = 1.5` separates it. Separable.
- **OR**: (0,0) is the only 0. A line like `x1 + x2 = 0.5` separates it. Separable.
- **XOR**: (0,1) and (1,0) are 1, while (0,0) and (1,1) are 0. The 1s sit on opposite corners, so no single straight line can split them. **Not separable.**

Exercises from the slides (define the function yourself, then check separability):

- **Majority function** [my answer]: output 1 if more than half the inputs are 1. It is separable. Use all weights 1 and threshold just above n/2 (for 3 inputs, `x1 + x2 + x3 >= 2`).
- **Odd/Even (parity) problem** [my answer]: output 1 if the number of 1s is odd. Not separable. With 2 inputs it is exactly XOR, and more inputs only make it worse.

### 6.8 Limitation of the perceptron (Part 4)

- A perceptron (input layer straight to output layer) for XOR **cannot be derived by training**, because XOR's input patterns are not linearly separable.
- A **Multilayer Perceptron** (input layer, middle layer, output layer) **can** represent XOR.

### 6.9 Multilayer perceptron for XOR (weights from the slide)

- Hidden unit 1: weights 1 (from x1) and 1 (from x2), `θ = 1.2`. Fires only when both are 1, so it acts as **AND**.
- Hidden unit 2: weights 1 and 1, `θ = 0.4`. Fires when either is 1, so it acts as **OR**.
- Output unit: weight **-0.2** from hidden 1 and **0.6** from hidden 2, `θ = 0.5`.

| x1 | x2 | H1 net (θ 1.2) | H1 | H2 net (θ 0.4) | H2 | Out net = -0.2 H1 + 0.6 H2 | Out (θ 0.5) |
|---|---|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 0 | 0.0 | 0 |
| 0 | 1 | 1 | 0 | 1 | 1 | 0.6 | 1 |
| 1 | 0 | 1 | 0 | 1 | 1 | 0.6 | 1 |
| 1 | 1 | 2 | 1 | 2 | 1 | 0.4 | 0 |

So XOR = "OR but not AND". The negative weight from the AND unit pulls the output below threshold when both inputs are on.

---

## 7. The Backpropagation Network (Part 4, plus Part 5 extras)

### 7.1 What it is

- The **Backpropagation algorithm** is the training algorithm for a **multilayer perceptron**.
- **Feedforward**: apply the input at the input layer, compute the hidden layer outputs, then the output layer outputs.
- **Backward error propagation**: compute errors and adjust weights going backward, from output to hidden layer to input layer.
- A **Back-Propagation Neural Network (BPN)** is an instance of a multilayer neural network with an input layer, a hidden layer, and an output layer. Each layer has its own job.

### 7.2 The layers

- **Input layer**: accepts inputs from the outside world and passes them to every hidden neuron. Input units **do not process** the inputs.
- **Hidden layer**: detects the **features hidden** in the input patterns. The output layer uses these features to decide the output pattern.
- **Output layer**: takes the hidden layer's signals and sets the output pattern of the whole network.

### 7.3 Training algorithm (4 steps)

1. **Initialization**: set all weights and thresholds to random numbers.
2. **Activation**: apply inputs (x pattern) and desired outputs (y pattern).
   - Hidden outputs: `Yh(p) = sigmoid( sum of x_i * w_hi - θ_h )`
   - Output layer outputs: `Yo(p) = sigmoid( sum of y_h * w_oh - θ_o )`
3. **Weight training**: update weights by propagating backward the errors of the output neurons.
4. **Iteration**: repeat from step 2 until the error criterion is met. The sum of the errors in the output units for an epoch should not exceed a set value (a very small number).

### 7.4 Simplified BPN algorithm (the 10 steps, slide notation)

Notation: superscript h = hidden layer, o = output layer. p = the training pattern. N = number of input units, L = number of hidden units, M = number of output units. η (eta) = learning rate.

1. Apply the input vector `x_p = (x1, x2, ..., xN)` to the input units.
2. Net input to each hidden unit j:
   ```
   Net^h_pj = sum over i=1..N of ( w^h_ji * x_i ) - θ^h_j
   ```
3. Output of each hidden unit:
   ```
   I_pj = f^h_j( Net^h_pj )
   ```
4. Net input to each output unit k:
   ```
   Net^o_pk = sum over j=1..L of ( w^o_kj * I_pj ) - θ^o_k
   ```
5. Output of each output unit:
   ```
   O_pk = f^o_k( Net^o_pk )
   ```
6. Error term for each output unit:
   ```
   δ^o_pk = ( Y_pk - O_pk ) * f^o'_k( Net^o_pk )
   ```
7. Error term for each hidden unit:
   ```
   δ^h_pj = f^h'_j( Net^h_pj ) * sum over k=1..M of ( δ^o_pk * w^o_kj )
   ```
   The hidden error terms are computed **before** the output weights are updated (use the old output weights).
8. Update output layer weights:
   ```
   w^o_kj(t+1) = w^o_kj(t) + η * δ^o_pk * I_pj
   ```
9. Update hidden layer weights:
   ```
   w^h_ji(t+1) = w^h_ji(t) + η * δ^h_pj * x_i
   ```
   The order of the weight updates within a layer does not matter.
10. Repeat from step 1 until the error is acceptably small for every training pair. Error term on the slide:
   ```
   E_p = 1/2 * sum over k=1..M of ( δ^o_pk )^2
   ```
   [general knowledge] Most textbooks write `E_p = 1/2 * sum of (Y_pk - O_pk)^2`. Use the slide's version on the exam unless told otherwise, and know the textbook one.

Symbols:

- `x_i`: input i. `Y_pk`: desired output of output unit k. `O_pk`: actual output of output unit k.
- `I_pj`: actual output of hidden unit j.
- `w^h_ji`: weight from input i to hidden unit j. `w^o_kj`: weight from hidden j to output k.
- `θ`: threshold of a unit (subtracted from the net input).
- `f'`: derivative (slope) of the output function.
- `δ`: error term of a unit, how much that unit is to blame for the error.

### 7.5 Sigmoid and its derivative

```
f(x)  = 1 / (1 + e^-x)
f'(x) = f(x) * ( 1 - f(x) )
```

So for an output unit, `f^o'(Net^o_pk) = O_pk * (1 - O_pk)`. For a hidden unit, `f^h'(Net^h_pj) = I_pj * (1 - I_pj)`. This is why the sigmoid is handy: the slope comes straight from the output you already computed. [The derivative identity is general knowledge. The slides use `f'` without writing it out.]

So the working formulas become:

```
δ^o_pk = ( Y_pk - O_pk ) * O_pk * ( 1 - O_pk )
δ^h_pj = I_pj * ( 1 - I_pj ) * sum over k of ( δ^o_pk * w^o_kj )
```

### 7.6 Fully worked example (the Part 4 BPN problem)

**Architecture**: 2 input units, 3 hidden units, 2 output units, fully connected.

**Weight naming on this problem**: `Wh_ij` = weight from **input j to hidden i**. `Wo_ij` = weight from **hidden j to output i**.

**Initial weights:**

```
Wh11 = 0.30   Wh12 = 0.20      (into hidden 1, from input 1 and input 2)
Wh21 = 0.30   Wh22 = 0.20      (into hidden 2)
Wh31 = 0.30   Wh32 = 0.20      (into hidden 3)

Wo11 = 0.35   Wo12 = 0.25   Wo13 = 0.25   (into output 1, from hidden 1, 2, 3)
Wo21 = 0.20   Wo22 = 0.25   Wo23 = 0.15   (into output 2, from hidden 1, 2, 3)
```

**Parameters:** learning rate `η = 0.15`. All thresholds 0 (`θh1 = θh2 = θh3 = θo1 = θo2 = 0.0`). Sigmoid `f(x) = 1/(1+e^-x)` on every hidden and output unit.

**Training pair:** input (1, 1), desired output (1, 0).

#### 1.1 Forward pass: net input and output of every unit

Hidden layer:

```
Net_h1 = Wh11*x1 + Wh12*x2 - θh1 = 0.30(1) + 0.20(1) - 0 = 0.50
Net_h2 = Wh21*x1 + Wh22*x2 - θh2 = 0.30(1) + 0.20(1) - 0 = 0.50
Net_h3 = Wh31*x1 + Wh32*x2 - θh3 = 0.30(1) + 0.20(1) - 0 = 0.50

I1 = f(0.50) = 1 / (1 + e^-0.5) = 1 / (1 + 0.606531) = 0.622459
I2 = 0.622459
I3 = 0.622459
```

Output layer:

```
Net_o1 = Wo11*I1 + Wo12*I2 + Wo13*I3 - θo1
       = (0.35 + 0.25 + 0.25) * 0.622459 = 0.85 * 0.622459 = 0.529090

Net_o2 = Wo21*I1 + Wo22*I2 + Wo23*I3 - θo2
       = (0.20 + 0.25 + 0.15) * 0.622459 = 0.60 * 0.622459 = 0.373476

O1 = f(0.529090) = 0.629271
O2 = f(0.373476) = 0.592299
```

#### 1.2 Error terms of every output and hidden unit

Output units (desired Y1 = 1, Y2 = 0):

```
f'(Net_o1) = O1(1 - O1) = 0.629271 * 0.370729 = 0.233289
f'(Net_o2) = O2(1 - O2) = 0.592299 * 0.407701 = 0.241481

δo1 = (Y1 - O1) * f'(Net_o1) = (1 - 0.629271) * 0.233289 =  0.370729 * 0.233289 =  0.086487
δo2 = (Y2 - O2) * f'(Net_o2) = (0 - 0.592299) * 0.241481 = -0.592299 * 0.241481 = -0.143029
```

Hidden units (using the **old** output weights):

```
f'(Net_h) = I(1 - I) = 0.622459 * 0.377541 = 0.235004   (same for all three)

Sum for h1 = δo1*Wo11 + δo2*Wo21 = 0.086487(0.35) + (-0.143029)(0.20) = 0.030270 - 0.028606 =  0.001665
Sum for h2 = δo1*Wo12 + δo2*Wo22 = 0.086487(0.25) + (-0.143029)(0.25) = 0.021622 - 0.035757 = -0.014135
Sum for h3 = δo1*Wo13 + δo2*Wo23 = 0.086487(0.25) + (-0.143029)(0.15) = 0.021622 - 0.021454 =  0.000167

δh1 = 0.235004 *  0.001665 =  0.000391
δh2 = 0.235004 * -0.014135 = -0.003322
δh3 = 0.235004 *  0.000167 =  0.000039
```

Error for this pattern (slide formula): `E_p = 1/2 (0.086487^2 + 0.143029^2) = 0.013969`. [general knowledge] With the textbook formula: `1/2 (0.370729^2 + 0.592299^2) = 0.244129`.

#### 1.3 New connection weights

Output layer, `Wo_kj(new) = Wo_kj + η * δo_k * I_j`:

```
η * δo1 * I = 0.15 *  0.086487 * 0.622459 =  0.008075
η * δo2 * I = 0.15 * -0.143029 * 0.622459 = -0.013354
```

| Weight | Old | Change | New |
|---|---|---|---|
| Wo11 | 0.35 | +0.008075 | 0.358075 |
| Wo12 | 0.25 | +0.008075 | 0.258075 |
| Wo13 | 0.25 | +0.008075 | 0.258075 |
| Wo21 | 0.20 | -0.013354 | 0.186646 |
| Wo22 | 0.25 | -0.013354 | 0.236646 |
| Wo23 | 0.15 | -0.013354 | 0.136646 |

Hidden layer, `Wh_ij(new) = Wh_ij + η * δh_i * x_j` (both inputs are 1):

```
η * δh1 = 0.15 *  0.000391 =  0.0000587
η * δh2 = 0.15 * -0.003322 = -0.0004983
η * δh3 = 0.15 *  0.000039 =  0.0000059
```

| Weight | Old | Change | New |
|---|---|---|---|
| Wh11 | 0.30 | +0.000059 | 0.300059 |
| Wh12 | 0.20 | +0.000059 | 0.200059 |
| Wh21 | 0.30 | -0.000498 | 0.299502 |
| Wh22 | 0.20 | -0.000498 | 0.199502 |
| Wh31 | 0.30 | +0.000006 | 0.300006 |
| Wh32 | 0.20 | +0.000006 | 0.200006 |

What happened in plain terms: output 1 was too low (0.63, wanted 1), so weights into it went up. Output 2 was too high (0.59, wanted 0), so weights into it went down. Hidden 2 feeds output 2 the hardest (0.25), so it took the biggest share of blame and its weights went down. Hidden weight changes are tiny because the error is diluted by the sigmoid slope twice.

Thresholds stayed fixed here. Part 5 notes that thresholds can be trained like weights for faster convergence [general knowledge: `θ(new) = θ - η * δ`, because θ is subtracted from the net input].

### 7.7 Delta rule and naming (Part 5, final slides)

- The **Delta Rule** is a further variation of Hebb's Rule, and it is commonly used.
- It keeps changing the input connection strengths to **reduce the difference (the delta)** between the desired output and the actual output.
- It changes weights in the way that **minimizes the mean squared error** of the network.
- The error is back propagated into previous layers one layer at a time, until the first layer is reached.
- Other names: **Widrow-Hoff learning rule** (spelled "Windrow-Hoff" on the slide) and **Least Mean Square (LMS) learning rule**.
- A network that follows the delta rule is called a **Feed Forward Network**.
- The Backpropagation Network gets its name from how it computes the error terms: the output units' error terms are propagated backward to the hidden units.

### 7.8 Pointers for the BPN (Part 5)

- You do not have to use all your data for training. A subset is often enough. Use the rest to check that the network does the right **mapping** on inputs it never saw.
- If the BPN will run in a **noisy** environment (example: pixel image to ASCII), include noisy inputs in training. Noisy patterns help convergence even when no noise is expected.
- The BPN is good at **generalization**. Given different inputs of the same class, it learns to key off the important similarities. Example: classifying 5-bit binary numbers as even or odd, training makes the least significant bit the only basis.
- The BPN does **not extrapolate** well. Training data should cover the whole expected **input space**.
- Pick training pairs **randomly**. Do not train fully on one class and then switch to another, or the network will **forget** the earlier training.
- With a sigmoid, outputs **never reach exactly 0 or 1**. Scale targets, for example 0.1 and 0.9 for the smallest and largest values.
- You can change the slope of the sigmoid by putting a constant in the exponent (`e^-2net`).
- Generally **3 layers** are enough: input, 1 hidden, output. Input and output sizes come from the application (input pattern, number of classes).
- There is no set rule for the number of hidden units. Use as few as possible because each one adds computing load. If the network does not converge, it may need more.
- Initialize weights to random values **between -0.5 and 0.5**. Thresholds (bias) can start small. Training thresholds too may speed up convergence.
- The **learning rate η** matters a lot. Keep it small, **0.05 to 0.25**, so the network settles. Smaller η means more iterations. You can change η as the error drops to speed things up.
- **Momentum**: add a fraction of the previous weight change to the current one to speed up convergence:
  ```
  w^o_kj(t+1) = w^o_kj(t) + η * δ^o_pk * I_pj + α * Δw^o_kj(t-1)
  ```
  `α` = momentum parameter, `Δw(t-1)` = the previous weight change.
- **Local minimum**: convergence can be **global** (wanted) or **local**. A local minimum is when the network stops learning before reaching an acceptable answer. Fixes: change the number of hidden nodes, change the learning parameters, or start over with different initial weights.
- Hypothetical error surface: x-axis is weights, y-axis is sum of errors. Training moves downhill on this surface.

### 7.9 Applications of ANN (Part 5)

- **Classification**: use inputs to decide the class. Is the input the letter A? Is the blob in the video a plane, and what kind? Is the word a future tense verb?
- **Prediction**: use inputs to predict an output. Pick stocks, predict weather, identify people with cancer risk.
- **Data association**: like classification, but also recognizes data with errors. Identify scanned characters and also notice when the scanner is not working.
- **Data conceptualization**: analyze inputs to find grouping relationships. Pull from a database the names of people most likely to buy a product.
- **Data filtering**: "smoothen" a signal. Take the noise out of a telephone signal.

---

## 8. Likely Exam Questions / Quick Self-Check

### Naive Bayes

1. Write Bayes' rule and say which part is the prior, the likelihood, and which part is dropped. (`P(L|W) = P(L) P(W|L) / P(W)`, drop `P(W)`.)
2. Why is it called "naive"? (Assumes words are independent given the language.)
3. What features does the instructor's sheet use? (The top 20 whole words per language plus an Others bucket.)
4. What is add-one smoothing and why is it needed? (Add 1 to every count so unseen words do not make the product 0.)
5. Compute `P(Tagalog)`. (109,854 / 784,438 = 0.140042.)
6. Compute `P(sa | Cebuano)`. (30,850 / 104,478 = 0.295278.)
7. Compute `P(unseen word | Ilocano)`. (1 / 101,190 = 0.0000098824.)
8. Classify "ang lahat ay nakapangyayari". (Tagalog.)
9. Classify "pagkamaopay hito nga sumal". (Waray, Hiligaynon close second.)
10. Why does a sentence with no known words go to Kapampangan? (Smallest total, so the largest `1/total`, which outweighs its smaller prior.)
11. Define precision, recall, accuracy with formulas.

### McCulloch & Pitts

12. List the five assumptions of the M&P model.
13. How many excitatory inputs fire a neuron? (At least 2.) What does one inhibitory input do? (Absolutely blocks firing.)
14. Draw and write the expression for precession, disjunction, conjunction, conjoined negation.
15. Give `N3(t)` and `N4(t)` for example 5. (`N1(t-1) v [N2(t-3) & ¬N2(t-2)]` and `N2(t-1) & N2(t-2)`.)
16. Build a network for `N3(t) := N2(t-2) & N1(t-3)`.
17. Two deficiencies of the M&P model? (How connections form through learning, and how to get fault tolerance.)
18. State Hebb's postulate (1949) and how ANNs implement it (adjusting weights).

### Perceptron

19. Write the net input and binary threshold output formulas.
20. How do you remove the threshold from a network? (Add a constant input of 1 whose weight carries the threshold.)
21. List the 5 steps of the perceptron learning procedure.
22. Write the weight update rule and name each symbol.
23. Continue a training table for a few rows given starting weights, θ, and r.
24. Final weights of the OR perceptron in the slides? (0.5 and 0.5 with θ = 0.5, after 6 epochs.)
25. When is convergence guaranteed? (Linearly separable input.) Define linear separability.
26. Show AND and OR are separable and XOR is not.
27. Is the majority function separable? (Yes.) Odd/even? (No.)
28. Why can't a perceptron learn XOR, and what fixes it? (Not separable. A multilayer perceptron with a middle layer.)
29. Verify the slide's XOR network on all 4 inputs.

### Backpropagation

30. Name the 3 layers and what each does.
31. List the 4 steps of BPN training (initialization, activation, weight training, iteration).
32. What does "feedforward" vs "backward error propagation" mean?
33. Write the sigmoid and its derivative.
34. Write the output and hidden error term formulas.
35. Why are hidden error terms computed before the output weights are updated?
36. Write the weight update formulas for both layers.
37. Redo the Part 4 problem from scratch: nets 0.5 (all hidden), outputs 0.622459, `Net_o1 = 0.529090`, `Net_o2 = 0.373476`, `O1 = 0.629271`, `O2 = 0.592299`, `δo1 = 0.086487`, `δo2 = -0.143029`, `δh = 0.000391, -0.003322, 0.000039`, new `Wo11 = 0.358075`.
38. What is momentum and what is its formula?
39. What is a local minimum and three ways out of it?
40. Recommended range for η and for initial weights? (0.05 to 0.25, and -0.5 to 0.5.)
41. Other names for the delta rule? (Widrow-Hoff, Least Mean Square.)
42. Five application types of ANN with one example each.

### General NLP

43. Define NLP (course definition).
44. NLU vs NLG, and the five NLU sub-areas.
45. Name the core NLP tasks and define three.
46. Give the NLP spectrum in order.
47. List the linguistic levels (terminologies).
48. Name the course's NLP areas of study and its stated goals.

### Pocket formulas

```
Naive Bayes:   score(L) = P(L) * product of P(wi | L)
               P(L) = total(L) / grand total,  P(w | L) = (freq + 1) / total(L)

Perceptron:    Net = sum W*X,   Y = 1 if Net >= θ else 0
               W(t+1) = W(t) + r * (Yc - Y) * X

Sigmoid:       f(x) = 1 / (1 + e^-x),   f'(x) = f(x)(1 - f(x))

BPN:           δo_k = (Y_k - O_k) * O_k(1 - O_k)
               δh_j = I_j(1 - I_j) * sum_k δo_k * Wo_kj
               Wo_kj += η * δo_k * I_j
               Wh_ji += η * δh_j * x_i
               E_p = 1/2 * sum (δo_k)^2   (slide)
```
