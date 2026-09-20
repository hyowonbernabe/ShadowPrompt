# CS 412: Information Assurance and Security

Source: CS 412 Notes 1.pdf (Prelim, "An Introduction")

## The Numbers: Cyberattacks and Data Breaches

- 80% of firms saw an increase in cyberattacks in 2020. Coronavirus alone blamed for a 238% rise in cyberattacks on banks. Phishing attacks rose 600% since end of February (2020).
- Ransomware attacks rose 148% in March 2020. Average ransomware payment rose 33% to $111,605 versus Q4 2019.
- 81 global firms from 81 countries reported data breaches in the first half of 2020.
- Worldwide information security market forecast to reach $170.4 billion in 2022 (Gartner).
- 95% of cybersecurity breaches are caused by human error.
- 88% of organizations worldwide experienced spear phishing attempts in 2019.
- 68% of business leaders feel their cybersecurity risks are increasing.
- On average, only 5% of companies' folders are properly protected.
- Data breaches exposed 36 billion records in the first half of 2020.
- 86% of breaches were financially motivated, 10% motivated by espionage.
- 45% of breaches featured hacking, 17% involved malware, 22% involved phishing.
- Top malicious email attachment types: .doc and .dot (37% combined), then .exe (19.5%).
- An estimated 300 billion passwords are used by humans and machines worldwide.
- Average time to identify a breach in 2020: 207 days. Average lifecycle of a breach (identification to containment): 280 days.
- Personal data was involved in 58% of breaches in 2020.
- Security breaches increased 11% since 2018, and 67% since 2014.
- Uber tried to pay off hackers to delete stolen data of 57 million users and keep the breach quiet.
- In 2018, an average of 10,573 malicious mobile apps were blocked per day.
- 94% of malware is delivered by email. 48% of malicious email attachments are office files.
- Top 5 business fears (IT Security Risks Survey 2017): inappropriate sharing of data via mobile devices (47%), physical loss of mobile devices exposing the organization to risk (46%), inappropriate IT resource use by employees (44%), incidents affecting suppliers that share data (43%), incidents involving non-computing connected devices (43%).
- Types of security events experienced (Kaspersky, proportion of businesses): viruses and malware 49% (+11%), inappropriate IT resource use by employees 39%, physical loss of devices/media containing data 33%, physical loss of mobile devices 32%, inappropriate sharing of data via mobile devices 31%, targeted attacks 27% (+6%), DDoS attacks 25%, electronic leakage of data from internal systems 25%, incidents involving non-computing connected devices 24%, incidents affecting third-party cloud services 24%, incidents affecting suppliers 24%, incidents affecting IT infrastructure hosted by another party 24%, incidents affecting virtualized environments 24%.

## High-Profile Cyberattacks and Data Breaches

Presented "in no particular order on scope, relevance, or impact." Theme: "If it can happen to them, it can happen to you."

### Bangladesh Bank Hacking
- Dridex malware installed in Bangladesh Central Bank's system around January 2016, gathered info on operational procedures for international payments/fund transfers (SWIFT).
- $20 million transfer meant for Shalika Foundation was flagged due to a spelling error, raising suspicion at Deutsche Bank.
- $81 million was sent to five accounts at RCBC (a Philippine foreign exchange broker); Bangladesh requested a freeze on transfers during Chinese New Year.
- Suspected to be state-funded hackers from North Korea.

**What is SWIFT?** Society for Worldwide Interbank Financial Telecommunications: a member-owned cooperative providing safe, secure financial transactions for members. Lets individuals/businesses make electronic/card payments even across different banks. Assigns each member institution a unique ID code (bank, country, city, branch). It is a messaging network using standardized codes; it does NOT hold or transfer assets itself, only facilitates secure communication between institutions.

### DNC Email Server Hacking
- Collection of 19,252 emails and 8,034 attachments from the Democratic National Committee (DNC), leaked and published by WikiLeaks on July 22, 2016.
- Emails included DNC staff's "off-the-record" correspondence with media personalities/reporters.
- Leaked first by DCLeaks (June/July 2016), then WikiLeaks (July 22, 2016), just before the 2016 Democratic National Convention.
- Caused allegations of bias against Bernie Sanders's campaign, contradicting DNC's stated neutrality.
- A hacker calling themselves Guccifer 2.0 claimed responsibility.
- Believed to have contributed to Donald Trump's win in the 2016 election.
- Two Russian intelligence groups were suspected.

### Yahoo! Email Hacking
- Half a billion accounts hacked in 2014; more than a billion accounts in 2013.
- Hackers used forged credentials tricking Yahoo servers into recognizing them as logged-in account holders ("cookie minting"/forged cookies) to read contents of about 6,500 accounts without needing username/password.
- A forged cookie is a maliciously crafted HTTP cookie impersonating a valid user without knowing their password; it can bypass authentication (even multifactor) no matter how complex the password is.
- Yahoo was criticized for late disclosure.
- Stolen data was sold on the dark web by an actor called "Peace."
- Believed to be state-sponsored. Considered the largest security breach in internet history as of 2014.

### Software AG Ransomware Attack
- Software AG is the second-largest software vendor in Germany, seventh-largest in Europe. Hit by ransomware in October 2020.
- Attacked by Clop ransomware; attackers demanded more than $20 million ransom.
- Disrupted part of internal network but cloud-based customer services remained unaffected. Negotiation attempts failed.

### Twitter Data Breach
- Breached in July 2020 by three individuals via a social engineering attack (phone phishing) that stole employee credentials and accessed internal management systems.
- Dozens of high-profile accounts hijacked, including Barack Obama, Jeff Bezos, Elon Musk.
- Attackers tweeted bitcoin scams earning over $100,000.
- DoJ charged 17-year-old Graham Ivan Clark as an adult as the alleged mastermind.
- Targeted about 130 accounts, netted about $121,000 in Bitcoin across nearly 300 transactions.

### WannaCry Ransomware Cyberattack
- Targeted Windows-based PCs, encrypted computer data.
- Ransom demanded in bitcoin, $300 to $600.
- Infected computers worldwide (e.g., NHS, Telefonica) that used Windows XP.
- Stopped by security researcher/blogger MalwareTech.
- Propagated via the EternalBlue Windows exploit, originally discovered by the NSA and used as an offensive weapon.
- Human factor: many companies had not patched systems even two months after Microsoft released the fix. Non-IT personnel with local admin rights sometimes disabled security software, letting infections spread across corporate networks.

### US Colonial Pipeline Ransomware Attack
- Colonial Pipeline halted systems across its 5,500 miles of pipeline after a ransomware attack.
- US officials attributed it to a criminal group called "DarkSide," not a nation-state.
- Comes amid broader concern about attacks on critical infrastructure, including breaches attributed to Russian intelligence and Chinese hackers on Microsoft systems.

### Comelec Data Breach Allegations 2022
- Manila Bulletin reported claims that COMELEC (Philippines Commission on Elections) servers were breached on January 8, with attackers downloading more than 60 GB of data.
- Data allegedly included usernames/PINs of vote-counting machines, network diagrams, IP addresses, privileged user lists, domain admin credentials, password lists, domain policies, ballot handling dashboard access, and QR code captures with logins.
- Group reportedly sold the info for P60 million and threatened to release the 60 GB dataset and claimed ability to manipulate election results if ignored.

### Future Cybersecurity Threats
- Threats have evolved from targeting individuals (private photos, bank accounts, credit cards) to targeting corporations (stealing/selling user data, corrupting databases) to targeting nations (cyberwar).
- The biggest future threats are expected to target nations rather than individuals or corporations.

## Your Job as a Computing Security Professional

- Need a clear understanding of the progression and use of data within the organization to design/implement appropriate business systems (databases, networks, applications).
- Roles include: monitoring network usage for policy compliance, keeping up to date with IT security standards/threats, performing penetration tests to find flaws, collaborating with management/IT to improve security.
- Two fields:
  - **Information Assurance**: focuses on ensuring availability, integrity, authentication, confidentiality, and non-repudiation of information and systems. Includes restoration of systems via protection, detection, and reaction capabilities. (Management aspect.)
  - **Information Security**: centers on protecting information/systems from unauthorized access, use, disclosure, disruption, modification, or destruction, to provide confidentiality, integrity, and availability. (Technological aspect.)

## Cyberwarfare

- **Cyberwarfare**: use of cyberattacks against an enemy state, causing harm comparable to actual warfare and/or disrupting vital computer systems. Intended outcomes: espionage, sabotage, propaganda, manipulation, or economic warfare. Some governments treat it as part of military strategy. Controversial whether it can truly be called "war."

### Case of Iran's Nuclear Facilities (Stuxnet)
- Over fifteen Iranian facilities attacked/infiltrated by the Stuxnet worm.
- Suspected origin: a USB drive inserted by a worker at the Natanz nuclear facility in 2010.
- IAEA inspectors observed an unusual number of uranium-enriching centrifuges breaking, cause unknown at the time.
- Iran later contracted computer security specialists in Belarus, who discovered the Stuxnet worm on Iranian systems.
- Estimated Stuxnet destroyed 984 uranium-enriching centrifuges, about a 30% decrease in enrichment efficiency.

### The Stuxnet Worm (technical detail)
- Originally aimed at Iran's nuclear facilities; later mutated and spread to other industrial/energy facilities.
- Targeted programmable logic controllers (PLCs) used to automate machine processes.
- Discovered in 2010; first known virus capable of crippling hardware. Reportedly created by the US NSA, the CIA, and Israeli intelligence.
- Destroyed centrifuges by causing them to burn themselves out. Later variants targeted water treatment plants, power plants, gas lines.
- Traveled via USB sticks, spread through Microsoft Windows computers, searched for Siemens Step 7 software (used for PLCs).
- Sent damage-inducing instructions to equipment while sending false feedback to the main controller, so monitors saw no problem until equipment self-destructed.

### What a Modern Cyberwarfare Looks Like
Cyberwarfare aims to destabilize or destroy critical systems to weaken a target country. Forms include:
1. Attacks on financial infrastructure
2. Attacks on public infrastructure (dams, electrical systems)
3. Attacks on safety infrastructure (traffic signals, early warning systems)
4. Attacks against military resources or organizations

### Cyber Warfare vs Cyber War
- Cyberwarfare = the techniques used while engaging in cyber war (e.g., a state-sponsored hacker attacking the Bank of England as an act of cyberwarfare within a broader cyber war against England and its allies).

### Types of Cyber Warfare
- **Espionage**: spying on another country to steal secrets, e.g. using a botnet or spear-phishing to gain a foothold before extracting sensitive info.
- **Sabotage**: assessing threats to identified sensitive data, including third parties, competitors, and insider threats (disgruntled/negligent employees).
- **Denial-of-Service Attack (DoS)**: flooding a website with fake requests so it can't serve legitimate users, potentially crippling critical sites used by citizens, military, safety personnel, scientists.
- **Electrical Power Grid attack**: could disable critical systems, cripple infrastructure, cause deaths, and disrupt communications (text messaging, telecom).
- **Propaganda**: trying to control hearts/minds of people in the targeted country, exposing embarrassing truths or spreading lies to reduce faith in their government.
- **Economic Disruption**: attacking computer networks of stock markets, payment systems, or banks to access funds or block the target's ability to fund itself.
- **Surprise Cyberattack**: massive strikes with an effect like Pearl Harbor or 9/11, catching the enemy off guard, possibly preceding a physical attack (hybrid warfare).

### Reasons / Motivations for Cyber Warfare
- **Military**: gaining control of key elements of an enemy's cyberspace can bring their military to its knees, securing an otherwise costly victory.
- **Civil**: attacking civil infrastructure directly impacts citizens, inspiring fear or revolt, weakening the opponent politically.
- **Hacktivism**: hackers using cyberattacks to promote an ideology, e.g. spreading propaganda or exposing secrets to weaken an opponent's world standing.
- **Income Generation**: cyber "soldiers" attacking for personal financial benefit, whether paid by a government or stealing directly from financial institutions.
- **Nonprofit Research**: stealing valuable research results (e.g., vaccine research) that another country needs.

### Russia-Ukraine Cyber Warfare in 2022
- Crisis began February 2022; also fought in cyberspace.
- New "viper" malware attacked Ukrainian targets, installed on several hundred machines.
- KillDisk and HermeticWiper malware used against Ukrainian organizations, designed to destroy data on devices.
- A fake copy of Remote Manipulator System (RMS), a remote-control utility tool, was distributed in Ukraine via fake "Evacuation Plan" emails.

### Prevention / Mitigation
- Good IT security practices: regular patches/updates, strong passwords, password management, identification and authentication software.
- Against Stuxnet-style attacks: virus scanning or banning of USB sticks/portable media, endpoint security software to intercept malware before it travels the network.
- Protecting industrial networks:
  - Separate industrial networks from general business networks with firewalls and a demilitarized zone (DMZ).
  - Closely monitor machines automating industrial processes.
  - Use **application whitelisting**: specifying an approved index of software/executables allowed to run, to protect against harmful applications.
  - Monitor and log all network activity.
  - Implement strong physical security (card readers, surveillance cameras).
- Organizational practices: routine auditing and third-party testing to be self-critical of defenses; develop an incident response plan (via a CERT, Computer Emergency Response Team) to react quickly and restore systems; train employees with simulated events and build a culture of security awareness (e.g., Red Team vs Blue Team exercises).

## Concepts and Characteristics of Information and Information Security

### Introduction
- Information systems are core infrastructure for commerce, banking, telecommunications, health care, national security, driving demand for information assurance/security specialists.
- **Cybersecurity**: securing transmission from sender to receiver plus physical awareness/security, encryption/decryption, verification procedures, countermeasures against cyberterrorism, cybercrime investigation, cyberbullying. Defends computers, servers, mobile devices, electronic systems, networks, data from malicious attacks. Also called information technology security or electronic information security. Common categories:
  - **Network security**: securing a computer network from intruders (targeted attackers or opportunistic malware).
  - **Application security**: keeping software/devices free of threats; a compromised application can expose the data it was meant to protect; security should begin at the design stage.
- **Information Security**: the state of being protected against unauthorized use of information (especially electronic data), or the measures to achieve this. Deals with verification, authorization, and secure distribution/transmission.
- **Information Assurance and Security (IAS)**: management and protection of knowledge, information, and data, ensuring it is verified, authorized, secure, readily available, and personal. Combines Information Assurance and Information Security. Both disciplines share concerns: risk management, cybersecurity, corporate governance, compliance, auditing, business continuity, disaster recovery, forensic science, security engineering, criminology.

### Information Assurance vs Information Security (restated)
- **Information Assurance**: ensures availability, integrity, authentication, confidentiality, non-repudiation of information/systems; includes restoration via protection, detection, reaction. More on the MANAGEMENT aspect.
- **Information Security**: protects information/systems from unauthorized access, use, disclosure, disruption, modification, destruction, to provide confidentiality, integrity, availability, via policy, training, awareness programs, and technology. More on the TECHNOLOGICAL aspect.

### Cybersecurity vs IAS
- All these fields share goals/principles but differ in focus.
- IAS falls under Cybersecurity and deals specifically with security of information.
- IAS is a more specialized field compared to general Cybersecurity or Networking.

### History of Information Security
- **1990s**: Internet made available to the public. Early connections relied on de facto standards (accepted in practice, no formal consensus process, often no public documentation, resulting from marketplace domination), which did little to ensure security. Early internet deployment treated security as low priority.
- **2000s**: information security became important for national defense.
- **1970s-80s**: ARPANET grew popular; potential for misuse grew. In 1973, Robert Metcalfe (developer of Ethernet) noted ARPANET security problems: insufficient controls at remote sites, vulnerable password structure/format, lack of safety procedures for dial-up connections, nonexistent user identification/authorization.
- **Morris Worm (November 1988)**: Robert Morris released a worm from a system at MIT, though he was at Cornell. Affected about 10% of internet-connected systems; attack lasted several days while networks connected to the NSFNet backbone were disconnected to restore/patch systems. (NSFNet, created by the National Science Foundation in the 1980s, succeeded ARPANET to unify regional/specialized networks.) Raised awareness of vulnerabilities and led to creation of a Computer Emergency Response Team (CERT) at Carnegie Mellon.
- **Mafiaboy Attack (February 2000)**: large-scale DDoS attack against prominent websites using the Stacheldraht tool via a controlled botnet. Both the Morris worm and Mafiaboy attack targeted weaknesses in Unix-like operating systems: Morris exploited easily-exploited Unix services; Mafiaboy exploited bugs in how network stacks were written within the OS.

### MULTICS
- Much early computer security research centered on MULTICS (Multiplexed Information and Computing Service), a mainframe time-sharing OS developed mid-1960s by General Electric, Bell Labs, and MIT.
- First OS to integrate security into its core functions.
- In mid-1969, developers Ken Thompson, Dennis Ritchie, Rudd Canaday, and Doug McIlroy created a new OS originally called Unics (a play on MULTICS), later renamed UNIX about 20 years later. Unix source code was made available to the public.
- **Minix**: created/released 1987 by Andrew Tanenbaum.
- **Linux**: developed by Linus Torvalds, released October 5, 1991.

### Linux
- A fractured collection of different distributions.
- **Distribution**: a collection of a kernel, userland, graphical interface, and package-management system.
- **Kernel**: interfaces with hardware to manage memory/file systems and ensure programs run.
- **Package management**: used to install software, developed by distribution maintainers (e.g., RedHat uses RPM, RedHat Package Manager).

### Open Source vs Closed Source
- **Open Source**: e.g. Linux, source code and programming text made available for anyone to inspect. Richard Stallman believed all source code should be available. "Gratis vs libre": free as in free speech, not free as in free beer. Gratis = without cost; libre = without restrictions.
- **Closed Source**: software developed by companies who charge for it (commercial software).

### Security Issues with Open Source
- Lack of processes to ensure code is written securely.
- No rigorous testing (usually done by the developer only); smaller open source projects lack dedicated testers that larger projects have.
- Source code sits in public repositories that could be replaced with modified versions containing back doors (mitigated by cryptographic hashing, but not foolproof).
- Less concerned with strict release schedules compared to commercial software (e.g. Microsoft batches updates), so fixes aren't guaranteed to be high quality.

### Linux Distributions
- **Slackware**: early distribution, popularity has waned.
- **RedHat**: popular; fragmented into RedHat Enterprise Linux (commercial/purchased) and Fedora (RedHat's development distribution).
- **Mint and Ubuntu**: derived from Debian (created by Ian Murdock).

### RAND R-609
- **Rand Report R-609** ("Security Controls for Computer Systems"): a DoD-sponsored paper attempting to define controls/mechanisms needed to protect a multilevel computer system.
- Classified for years, declassified October 10, 1975; considered the paper that started the study of computer security.
- By 1967, rapid acquisition of computer systems created a need for security; a task force was formed to study securing classified information systems, resulting in the recommendations that became R-609.
- Project RAND: an organization formed immediately after WWII (became independent nonprofit May 14, 1948) to connect military planning with R&D decisions. RAND is a contraction of "research and development."
- R-609 was a widely recognized published document identifying the role of management and policy issues in computer security; it noted military risks that routine practices could not mitigate. Signaled a pivotal moment including: securing the data, limiting random/unauthorized access to that data, and involving personnel from multiple organizational levels in information security matters.

### What is Security?
- Quality or state of being secure; being free from danger; being protected from adversaries or hazards.
- **National Security**: multilayered processes protecting the sovereignty of a state, its assets, resources, and people.
- **Role of management**: ensuring each strategy is properly planned, organized, staffed, directed, and controlled.

### Specialized Areas of Security
- **Physical security**: protecting people, physical assets, and the workplace from threats like fire, unauthorized access, and natural disasters.
- **Personnel security**: protecting individuals/groups authorized to access the organization and its operations.
- **Operations security**: securing the organization's ability to carry out operational activities without interruption or compromise.
- **Communications security**: protecting an organization's media, technology, and content, and the ability to use these tools to achieve objectives.
- **Network security**: protecting data networking devices, connections, and contents, and the ability to use the network for the organization's data communication function.

### Components of Information Security (diagram concept)
Information Security is composed of: Management of Information Security, Computer and Data Security, Network Security, and Policy (these overlap, with Information Security as the umbrella over all).

### The C.I.A. Triangle
Confidentiality, Integrity, Availability form a triangle representing the core principle of information security.
- The CIA triad is the most popular term for the principle of information security; some sources tie its credibility back to Julius Caesar (Roman politician).
- It is the most common key principle for building a security plan and appears in fundamental information security certifications like CISSP (Certified Information System Security Professional).

#### Confidentiality
- Information has confidentiality when protected from disclosure/exposure to unauthorized individuals or systems.
- Ensures only those with rights/privileges can access information; confidentiality is breached when unauthorized parties view information.
- Measures to ensure confidentiality: information classification, secure document storage, application of general security policies, education of information custodians and end users.
- Especially valuable for personal information about employees, customers, or patients. Organizations (government agencies like SSS, BIR, PhilHealth, or businesses) are expected to keep such info confidential.
- Consumers trade confidential info for convenience (online forms, loyalty cards — related concept: "salami theft").
- Example breach: employees throwing away documents with critical info without shredding.

#### Integrity
- Protection from unauthorized modification (add, delete, change) of data; ensures data can be trusted as accurate and not inappropriately modified.
- Considered the cornerstone of information security (corrupted information has little/no value).
- State of being "whole," complete, and uncorrupted.
- Threatened by corruption, damage, destruction, or disruption of authentic state; can occur during entry, storage, or transmission.
- Computer viruses and worms are designed to corrupt data.
- Key detection methods: changes in file size, hash value, checksum.
- Integrity also covers **non-repudiation**: assurance that someone cannot deny the validity of something; a legal concept providing proof of data origin and integrity.
- Causes of data corruption: faulty programming, noise in transmission channel/media.
- Compensation: error control techniques.

#### Availability
- Enables a user (person or computer system) to access information in a usable format without interference or obstruction.
- Protects the functionality of support systems, ensuring data is available at the point in time (or period) needed for decisions.
- Available to **authorized** users, e.g. research libraries requiring identification for access.

### Other Critical Characteristics of Information

- **Accuracy**: free from mistakes/errors, has the value the end user expects. If intentionally or unintentionally modified, it is no longer accurate (e.g. bank account errors, election data).
- **Authenticity**: quality of being genuine/original rather than a reproduction or fabrication; in the same state it was created. Assuming you know an email's origin is often wrong.
  - **Email spoofing**: sending an email with a modified field, often the originator's address.
  - **Phishing**: an attacker attempts to obtain financial or personal information via fraudulent means, often posing as another individual/organization. Pretending to be someone else is called **pretexting** when done by law enforcement or private investigators. Common target: e-commerce, bank, brokerage organizations, e.g. luring victims to a fake web server to steal account numbers/passwords.
  - **Common features of phishing emails**:
    - Too Good To Be True: lucrative/attention-grabbing offers (e.g. "you won an iPhone/lottery").
    - Sense of Urgency: pressure to act fast, threats of account suspension; legitimate organizations give ample time and don't ask for personal detail updates over the internet.
    - Hyperlinks: hovering reveals the actual URL, which may be disguised (e.g. "bankofarnerica.com" with an 'r'+'n' replacing 'm').
    - Attachments: unexpected/unexplained attachments often carry payloads like ransomware; .txt files are always safe to open.
    - Unusual Sender: anything unexpected, out of character, or suspicious from a known or unknown sender should not be clicked.
  - **Case example**: In 2006, Hewlett Packard CEO Patricia Dunn authorized contract investigators to use pretexting to "smoke out" a corporate director suspected of leaking confidential info. **Pretexting**: a social engineering attack creating a false situation/pretext to lure a victim into giving private information they wouldn't normally give outside that context. The move was ethically questioned and backfired, generating a firestorm of negative publicity for Dunn.
- **Privacy**: information collected/used/stored by an organization is intended only for the purposes stated by the data owner at collection time. As a characteristic of information, it doesn't mean freedom from observation, but that info will be used only in ways known to the person providing it.
  - Many organizations collect, swap, and sell personal information as a commodity.
  - Combining information from separate sources can build detailed databases usable in ways not agreed to or communicated to the original owner.
  - Many people are aware of these practices and look to government for protection.
- **Identification**: information possesses identification when it can recognize individual users; the first step in gaining access to secured material, serving as the foundation for subsequent authentication and authorization. Performed via username or other ID.
- **Authorization**: after a user's identity is authenticated, authorization assures the user (person or computer) has been specifically and explicitly authorized by the proper authority to access, update, or delete the contents of an information asset. Examples: access control lists/authorization groups in networking; database authorization schemes verifying specific permitted functions (reading, writing, creating, deleting).
- **Utility**: quality of having value for some purpose or end; information has value when it can serve a purpose. If available but not in a meaningful format for the end user, it is not useful (e.g. census data is useful differently to a private citizen versus a politician).
- **Possession**: quality/state of ownership or control. Information is in one's possession if obtained, regardless of format or other characteristics. A breach of possession does not always mean a breach of confidentiality (e.g. stealing encrypted customer data: possession is breached, but confidentiality may remain intact if the data can't be read).

## Information Security Management

### Components of an Information System (IS)
Diagram relationship: Information sits at the center, connected to Data, Hardware, People, Procedure (and implicitly network/software).
- **Software**: applications, operating systems, command utilities; the hardest component to secure. Buggy software, holes, and flawed system software (even in cars, smartphones) contribute to vulnerabilities.
- **Hardware**: physical technology housing software, storing/transporting data, and providing interfaces for entering/removing information.
- **Data**: an organization's most valuable asset, needing protection.
- **People**: can be the weakest link in an organization's information security.
- **Procedures**: written instructions for accomplishing information-security-related tasks.
- **Networks**: created much of the need for increased computer and information security.

### Asset (in the context of information security)
- Modern businesses process/store vast sensitive data, needed to provide services, improve user experience, or make better decisions.
- Organizations must protect this data; unauthorized access (cyberattack or privacy breach) causes lasting damage.
- **Assets**: data, devices, or other components of an organization's systems that are valuable, typically because they contain sensitive information or provide access to it. Information and its critical elements (plus the systems/hardware that use, store, transmit it) must be protected at all cost. Examples: desktop computers, laptops, company phones, and the applications running on them.

### Balancing Information Security and Access
- To achieve balance (satisfying both users/employees/customers and security professionals): "the security level must allow reasonable access, yet protect against threats."
- Imbalances occur if mishandled: too much free access is harmful, too much restriction is impractical; the exact required balance must be maintained so users and security professionals are both satisfied.
- From a management perspective, information security **cannot be absolute**; it is a **process, not a goal**. The organization must protect user interests while providing appropriate access, and must provide adequate security so not just anyone can access information. The need to balance security and accessibility arises because information security can never be absolute.

### Information Security Management
- A way of protecting an organization's sensitive data from various threats/vulnerabilities.
- Typically embedded via an **ISMS** (Information Security Management System), the framework for managing information security.
- At the center is information risk management: organizations assess risks and how they could compromise confidentiality, integrity, and availability of information.
- Managing risk gives organizations comprehensive understanding of ways they could suffer breaches/disruptive events and steps to protect themselves.
- Given growing threats of data breaches and regulatory action, effective information security risk management is essential.

### Information Security Risk Management
Prevention of data breaches begins with risk management: identifying information assets and how they can be compromised, splitting risk into components:
- **Vulnerabilities**: known flaws that can be exploited to damage/compromise sensitive information.
- **Threats**: the actions by which vulnerabilities are exploited (e.g. a cybercriminal leveraging a software flaw).
- **Likelihood**: how likely it is a vulnerability will be exploited.
- **Impact**: damage that occurs when a threat is exploited, encompassing delays, lost business, financial effects, and reputational damage.
- **Continuous monitoring**: after identifying information/assets, agencies should also identify the requirements/rules governing that information.

### Review of Management Concepts
- **Management**: the process of achieving objectives using a given set of resources.
- **Manager**: a member of the organization assigned to marshal/administer resources, coordinate task completion, and handle roles necessary to complete objectives. (An IT Security manager applies this specifically to information security.)

### Behavioral Types of Information Security Leaders
- **Autocratic**: reserves all decision-making for themselves, "I do as I say," does not accept alternative viewpoints. ("I make all the decisions regarding our information security.")
- **Democratic**: makes decisions by group consensus after accepting varying inputs, forming a majority-supported position. ("Let us work out an information security plan for our company.")
- **Laissez-faire**: "laid-back" leader who sits back and lets the process develop, making only minimal decisions. ("Let us wait for the security problems to occur and we will deal with it once we have them.")
(Discussion prompt in the source: which leadership type would work best for a typical organization and why.)

### Principles of Information Security Management: The Six Ps
The extended characteristics of information security management are known as the six Ps: **planning, policy, programs, protection, people, and project management**.
Strategy flow: Business Strategy -> IS/IT Strategy -> Information Security Strategy.

### Typical Procedure for Solving Security/Operational Problems
A basic blueprint methodology:
1. Recognize and Define the Problem
2. Gather Facts and Make Assumptions
3. Develop Possible Solutions
4. Analyze and Compare Possible Solutions
5. Select, Implement, and Evaluate a Solution

### Worked Example: Solving an Information Security Problem (spam email case)
1. A department complains about a large number of unsolicited commercial emails (spam); the Information Security (IS) Manager examines whether these are valid.
2. IS Manager interviews employees and makes assumptions (e.g. employees may have signed up on vendor sites requiring email sign-in, or employees are "misbehaving").
3. IS Manager leads brainstorming sessions to identify possible solutions; once the email source is traced, the IS manager can contact the email server admin and firewall admin.
4. Rank the solutions: for spam email, discard expensive options and narrow to:
   - Do nothing and accept it as a cost of doing business.
   - Have the System Admin change user email accounts.
   - Have the Firewall Admin filter access to and traffic from spam sites.
5. Evaluate the chosen solution and require employees to submit periodic reports.
