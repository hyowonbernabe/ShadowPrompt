# CS 412: Information Assurance and Security (Midterm Reviewer)
Instructor: Randy B. Domantay (rbdomantay@slu.edu.ph)

Exam: November 9, 2026. No coverage note and no exam type were given, so this covers all three midterm decks and is set up for both multiple choice and short answer or essay questions.

Sources: Midterms 1.pdf (Incident Response Plan and AI), Midterms 2.pdf (Red Team / Blue Team, Social Engineering, Zero Trust), Midterms 3.pdf (Malware, Other Forms of Attacks, Online Scams, Forms of Cyberattacks). The older file "CS 412 Midterms 2 Incident Response Plan and AI (1).pdf" is word for word the same as Midterms 1, so nothing extra came from it. Applied examples come from Midterm Activity 1 and our Midterm Project.

## Prelim Recap (only what the midterm builds on)

- **CIA triad**: the three goals of security. **Confidentiality** means only the right people can see data. **Integrity** means nobody changes it without permission. **Availability** means it is there when authorized users need it. A security incident is defined by these three (see Part 1).
- **Vulnerability**: a known flaw that can be exploited. **Threat**: the action that exploits it.
- **Phishing**: trying to get financial or personal info by posing as someone legitimate. **Pretexting**: making up a false situation to get a victim to give up private info. Both come back in the social engineering and malware sections.
- **Integrity check methods** from prelim (file size, hash value, checksum) tie into how antivirus spots infected files.

## Part 1: Incident Response Plan (IRP)

### Incident Response
- **Incident response** (also called cybersecurity incident response) is an organization's **processes and technologies** for detecting and responding to cyberthreats, security breaches, or cyberattacks.
- A formal incident response plan lets cybersecurity teams **limit or prevent damage**.

### Security Incidents
- A **security incident** (or security event) is any **digital or physical** breach that threatens the **confidentiality, integrity, or availability** of an organization's information systems or sensitive data.
- They range from **intentional** attacks by hackers or unauthorized users to **unintentional** violations of IT security policy by legitimate, authorized users.
- Most common security incidents (7):
  1. Ransomware
  2. Phishing and social engineering
  3. DDoS attacks
  4. Supply chain attacks
  5. Insider threats
  6. Privilege escalation attacks
  7. Man-in-the-middle (MITM) attacks

### Overview of the IRP
- Incident handling is normally guided by an incident response plan.
- Plans are created and executed by a **computer security incident response team (CSIRT)**, a team made up of stakeholders from across the organization.
- An **IRP** is a **written document, formally approved by the senior leadership team**, that helps the organization **before, during, and after** a confirmed or suspected security incident.
- The IRP clarifies **roles and responsibilities** and gives guidance on key activities.
- It should include a **cybersecurity list of key people** who may be needed during a crisis.

### Before a Cybersecurity Incident (9 things to do)
1. **Train the staff.** Everyone must know their role and how to report suspicious events. Be gracious about false alarms. Reward people who report, as part of a **culture of security**.
2. **Review your plan with an attorney.** Attorneys have preferences on how to deal with outside IR vendors, law enforcement, and other stakeholders.
3. **Meet your local anticybercrime police unit / NBI cybercrime group.** Do this in coordination with your attorney so you already know your contacts.
4. **Print the documents and contact list** and give a copy to everyone with a role. During an incident, internal email, chat, and document storage may be down.
5. **Develop an incident staffing and stakeholder plan.** Decide who plays what role, and who needs to be notified that people might forget (board of directors, key investors, critical partners).
6. **Review the plan quarterly.** The best IRPs are **living documents** that change as the business changes.
7. **Prepare press responses in advance.** Have a **holding statement** ready in case a reporter calls claiming to have stolen data.
8. **Select an outside technical resource or firm** to investigate potential compromises.
9. **Conduct an attack simulation exercise**, called a **tabletop exercise (TTX)**. A TTX is a roleplaying game where a **facilitator** presents a scenario to the team (for example, the head of communications gets an email from a reporter about rumors of a hack). The facilitator feeds in updates to see how everyone plays their role.


### During a Cybersecurity Incident (3 roles)

| Role | What they do | Key detail |
|---|---|---|
| **Incident Manager (IM)** | Leads the response. Manages communication flows, updates stakeholders, delegates tasks. | Does **not** do any technical work. Watches the clock because of **time dilation** (people lose track of time in a crisis). May lead the retrospective meeting. |
| **Tech Manager (TM)** | The **subject matter expert**. | Brings in other internal or external technical experts, with the IM's consent (and possibly the attorney's). |
| **Communications Manager (CM)** | Talks to reporters, posts updates on social media. | May deal with external stakeholders like shareholders. |

### After a Cybersecurity Incident
1. **Hold a formal retrospective meeting** (also called a **postmortem**). The IM reports the known incident timeline, asks for additions and edits, asks the team for analysis, and suggests areas for improvement.
2. **Update policies and procedures** based on the retrospective.
3. **Communicate the findings to staff.** Transparency builds trust and shows executives take security seriously. That is how you build a culture of security.

### What an IRP Includes (6 parts)
1. **Incident response playbook**: roles and responsibilities of each CSIRT member across the incident response lifecycle.
2. **Security solutions**: the software, hardware, and other technologies installed across the enterprise.
3. **Business continuity plan**: procedures for restoring critical systems and data as quickly as possible during an outage.
4. **Incident response methodology**: the specific steps at each phase, and who does them.
5. **Communications plan**: how to inform company leaders, employees, customers, and law enforcement.
6. **Instructions for collecting and documenting** incident info for postmortem review and, if needed, legal proceedings.

### Steps in Incident Response (NIST, as given in the slides)
The slides list **six** steps and label them NIST. Memorize these six in this order.

1. **Preparation**
2. **Detection and Analysis**
3. **Containment**
4. **Eradication**
5. **Recovery**
6. **Post Incident Review**

(Outside the slides: the official NIST SP 800-61 groups these into four phases, with containment, eradication, and recovery as one phase. The SANS framework also has six: Preparation, Identification, Containment, Eradication, Recovery, Lessons Learned. For this exam, answer with the slides' six.)

#### 1. Preparation
- The **first** phase and also a **continuous** one.
- The CSIRT picks the best procedures, tools, and techniques to respond, identify, contain, and recover quickly with minimal business disruption.
- Through **regular risk assessment**, the CSIRT identifies the business environment to protect, possible network vulnerabilities, and the types of incidents that pose a risk. It **prioritizes each incident type by potential impact**.

#### 2. Detection and Analysis
- The team **monitors the network** for suspicious activity. It analyzes data, notifications, and alerts from device logs and security tools (antivirus, firewalls).
- It **filters false positives** from real incidents and **triages** real alerts by severity. Triage means sorting by how urgent each one is.
- Most organizations use **SIEM** and **EDR** to monitor security events in real time and automate responses.
- The **communication plan** comes in here: once the CSIRT knows what kind of threat it is, it notifies the right people, then moves to the next stage.

#### 3. Containment
- Stop the breach or malicious activity from doing **further damage**. The emergency plans go into action.
- Two categories:
  - **Short-term mitigation** (quick action to stop the spread, like isolating an infected machine)
  - **Long-term containment** (more lasting measures while the fix is prepared)

#### 4. Eradication
- After containment, the team does full remediation and **complete removal of the threat**.
- Examples: removing malware, booting an unauthorized or rogue user off the network.
- The team reviews **both affected and unaffected systems** so no trace of the breach is left.

#### 5. Recovery
- Once the team is confident the threat is fully gone, it **restores affected systems to normal operations**.
- May involve deploying patches, rebuilding systems from backups, bringing systems back online.
- A **record of the attack and its resolution** is kept for analysis and improvement.

#### 6. Post Incident Review
- Through every phase, the CSIRT collects evidence and documents its steps.
- Now it reviews that info to understand the incident and gather **"lessons learned."**
- Goal: find the **root cause**, figure out how the attacker got in, and fix vulnerabilities so it does not happen again.

### Incident Response Technologies (6)

| Acronym | Full name | Plain meaning | Key trait to remember |
|---|---|---|---|
| **ASM** | Attack Surface Management | Finds, monitors, and shrinks all the exposed entry points across the digital environment. | Automates continuous discovery of vulnerabilities and attack vectors. Can **uncover previously unmonitored assets** and map relationships between assets. |
| **EDR** | Endpoint Detection and Response | Protects **endpoints** (laptops, desktops, servers, mobile devices). | Catches threats that **get past antivirus**. Collects data continuously from all endpoints and can respond automatically. |
| **SIEM** | Security Information and Event Management | **Aggregates and correlates** event data from many tools (firewalls, vulnerability scanners, threat intel feeds) and devices. | Fights **"alert fatigue"** by separating real threats from the flood of notifications. |
| **SOAR** | Security Orchestration, Automation and Response | Lets teams define **playbooks** (formal workflows that coordinate tools in response to incidents). | Can **automate** parts of those workflows. |
| **UEBA** | User and Entity Behavior Analytics | Uses behavioral analytics, machine learning, and automation to spot **abnormal user and device behavior**. | Best at catching **insider threats** and hackers using stolen insider credentials, since they mimic normal traffic. Often built into SIEM, EDR, and XDR. |
| **XDR** | Extended Detection and Response | **Unifies** security tools, control points, data, telemetry, and analytics across the hybrid IT environment into one central system. | Removes **silos** between tools. Helps overextended teams and SOCs (security operations centers) "do more with less" across the whole **kill chain** (the stages of an attack). |

Quick memory hooks: ASM = map the doors. EDR = guard each device. SIEM = collect and correlate logs. SOAR = run playbooks automatically. UEBA = spot weird behavior (insiders). XDR = tie everything together.

## Part 2: AI and Incident Response

### Key Benefits
- Quickly detect and control incidents **before escalation**.
- Helps mount a stronger defense, since attackers are also using AI.
- AI-powered systems improve IR through three things:
  1. **Faster detection of anomalies**
  2. **More proactive response processes**
  3. **Prediction of likely attack channels**

### Nine Areas Where AI Helps (each has two sub-points)

| Area | Sub-point 1 | Sub-point 2 |
|---|---|---|
| **Automated Threat Detection** | **Real-Time Monitoring**: analyzes traffic and logs live to find signs of malware, phishing, or data exfiltration (data being sneaked out). | **Behavioral Analysis**: watches for deviations from normal user behavior. Useful for **insider threats**. |
| **Threat Intelligence and Analysis** | **Data Correlation**: links unstructured data from threat feeds, news, and the dark web to spot emerging threats. Maps new vulnerabilities to affected systems. | **Malware Analysis**: auto-analyzes suspicious files or URLs. Reverse-engineers and classifies new variants faster. |
| **Incident Prioritization** | **Risk Scoring**: scores incidents by severity, impact, and likelihood of exploitation using past data and asset criticality. | **Automated Triage**: classifies incidents so the most important are handled first. Helps avoid **alert fatigue**. |
| **Incident Response Playbooks** | **Automated Playbook Execution**: triggers actions like isolating systems, blocking malicious IPs, disabling accounts. | **Adaptive Playbooks**: learns from past incidents and improves playbooks over time. |
| **Forensics and Root Cause Analysis** | **Data Mining**: digs through historical logs and system states to find the root cause, attack vector, timeline, and affected systems. | **Pattern Recognition**: finds trends and attacker tactics across incidents. |
| **Speed and Scalability** | **Scalable Analysis**: handles huge data volumes, for example a ransomware outbreak on thousands of endpoints. | **Reducing Time to Resolution**: automates repetitive tasks like log analysis to cut response time. |
| **Communication and Reporting** | **Automated Reporting**: builds reports with timelines, impact, recommendations, tailored to tech teams, management, or compliance officers. | **Collaboration**: real-time updates and suggestions keep the team aligned. |
| **Predictive Capabilities** | **Threat Forecasting**: predicts likely attack methods or targets using ML (machine learning). | **Vulnerability Management**: predicts which vulnerabilities are most likely to be exploited so they get patched first. |
| **Post-Incident Learning and Continuous Improvement** | **Incident Review and Analysis**: finds gaps in response time, mitigation, and controls. | **Training and Simulation**: creates realistic attack scenarios for practice. |

### Example: AI in a Ransomware Attack
1. Detects **unusual file encryption** across endpoints and **isolates** affected machines.
2. Identifies the **ransomware strain** from file signatures and behavior.
3. **Blocks the attacker's infrastructure** (IP addresses, domains).
4. Triggers **backup restoration**, notifies teams, generates reports for post-incident analysis.

### Limitations and Challenges (3)
1. **False Positives**: AI can flag things that are not threats, so humans must validate.
2. **Complexity**: setup needs expertise and continuous tuning.
3. **Human Oversight**: AI should **assist** experts, not replace them. Legal and ethical decisions still need human judgment.

## Part 3: Red Team / Blue Team Exercise

### Definition
- A **structured simulation** to test and improve an organization's defenses.
- **Red Team** = attackers. **Blue Team** = defenders.
- Purpose: find vulnerabilities, improve detection and response, strengthen overall security (physical or digital).

### Other Teams

| Team | Role |
|---|---|
| **Red** | Simulates attackers. |
| **Blue** | Defends. Detects and responds. |
| **White** | **Oversees** the exercise and makes sure the **rules are followed** (referee). |
| **Purple** | **Combines** Red and Blue efforts for **cooperative learning** and improvement. |

### Purpose (slide 2 list, 4 items)
1. **Testing Security Posture**: how well can the organization defend against different attacks.
2. **Improving Detection and Response**: Blue Team practices in real time.
3. **Identifying Weaknesses**: finds vulnerabilities the organization did not know about.
4. **Enhancing Communication**: better coordination between security teams and the rest of the organization.

### Purpose (slide 3 list, 4 items)
1. Identify weaknesses in cybersecurity defenses.
2. Improve incident detection and response.
3. Train the security team on real-world attack tactics and defense strategies.
4. Strengthen overall resilience to cyber threats.

### Two Categories of Red Team Attacks
- **Direct attacks**: go straight for the secret or try **brute force**.
- **Indirect attacks**: **trick people** into breaking protocol or exposing vulnerabilities (social engineering).
- Blue Teams try to reduce the risk of both.

### Red Team Responsibilities (5)
1. **Determine objectives**: pick target systems, networks, resources, or data.
2. **Exploit vulnerabilities**: use weaknesses in the tech stack to get unauthorized access.
3. **Compromise security**: use that access to hit the objective (steal info, enter a restricted physical area).
4. **Evade detection**: do it **without triggering alerts**.
5. **Develop report**: document findings and recommendations.

### Red Team Skillsets
- **Competitiveness**: want to win without getting caught.
- **Creativity**: new ways around controls and detections.
- **Cunning**: understand the psychology of social engineering.
- **Software development**: write their own tools and scripts, find code vulnerabilities.
- **System knowledge**: deep knowledge of systems, protocols, libraries, servers, tech trends.
- **Reverse threat engineering**: use info about known attacks to find adversary attack paths.
- **Penetration testing**: identify and exploit system and network vulnerabilities.
- **Research**: gather info on potential attacks to emulate them.
- Also listed: deep awareness of systems, protocols, and safeguards; strong development skills for custom tools that get around security; pen testing experience to avoid monitored activities; **social engineering skills** to manipulate people into sharing info or credentials.

### The Blue Team
- Made up of **incident response consultants** who guide the IT security team on where to improve against sophisticated attacks.
- The **IT security team** then maintains the internal network against risks.
- Prevention is often seen as the gold standard, but **detection and remediation are equally important**.
- **Breakout time**: the critical window between when an intruder compromises the **first machine** and when they can **move laterally** to other systems. Lateral movement means hopping from one system to others inside the network.
- **1-10-60 rule**: detect an intrusion in **under 1 minute**, assess its risk in **10 minutes**, eject the adversary in **under 60 minutes (1 hour)**.

### Blue Team Responsibilities (7)
1. **Education**: cybersecurity hygiene training against social engineering and physical attacks.
2. **Risk Analysis**: define critical assets, do risk assessments.
3. **Detection**: spot suspicious activity across networks, users, systems, devices.
4. **Investigation**: find exploitable vulnerabilities and respond to detections.
5. **Containment**: block red teamers from target systems and resources.
6. **Vulnerability scans**: scan for known or unknown vulnerabilities in OS, software, apps, firmware.
7. **Evidence collection and analysis**: gather and analyze forensic data like network traffic.

### Blue Team Skillsets
- **Organized**: manage data and follow procedures.
- **Meticulous**: detail-oriented, notices deviations from normal patterns.
- **Risk aware**: identifies risk and builds **threat profiles**.
- **Investigative**: uses threat intelligence to mitigate known risks and find new ones.
- **Technical hardening techniques**: fixes weaknesses to **reduce the attack surface**. Hardening means locking a system down.
- **Experience with detection systems**: traffic monitoring, firewall rules, packet filtering, SIEM.
- Full understanding of the organization's security strategy across people, tools, and technology.
- Analysis skills to prioritize the most dangerous threats.
- Hardening techniques especially around **DNS** to prevent phishing and web-based breaches.
- Awareness of the company's existing detection tools and alert mechanisms.

### High Level Ideas That May Emerge (from running the exercise)
- It is tough to cover every possible attack.
- It is **easier to think of attacks than protections**.
- Brainstorming attacks and protections feels disorganized.
- Both sides have open questions, or answers that start with "It depends."

### Benefits of a Red/Blue Team Exercise (5)
1. Identify **misconfigurations and coverage gaps** in security products.
2. Strengthen network security to detect targeted attacks and **improve breakout time**.
3. Raise **healthy competition** and foster cooperation between IT and security teams.
4. Raise staff awareness of **human vulnerabilities**.
5. Build skills and maturity in a **safe, low-risk training environment**.

## Part 4: The Psychology of Social Engineering

- **Social engineering psychology**: manipulating human behavior, emotions, and **cognitive biases** to trick people into giving up confidential info or doing things that hurt security.
- These attacks exploit **human nature**, not technical vulnerabilities.

### Seven Principles

| Principle | How it works | Slide example |
|---|---|---|
| **Authority** | People obey real or perceived authority figures (IT staff, executives, police). | Attacker impersonates a senior executive asking for sensitive data "urgently." |
| **Urgency and Scarcity** | Time pressure leads to quick, careless decisions. | Phishing email says the account will be suspended unless the password is updated now. |
| **Reciprocity** | People feel they must return a favor. | Scammer offers free software in exchange for email and password. |
| **Social Proof** | People follow what others are doing. | "Many colleagues already did this security update," so the target downloads the malicious attachment. |
| **Liking** | People are swayed by those they like (similarity, flattery, shared interests). | Attacker builds rapport through friendly talk and shared hobbies first. |
| **Commitment and Consistency** | Once people commit, they follow through to stay consistent. Starts small, escalates. | Short survey first, then requests for more sensitive data. |
| **Fear and Threats** | Fear of loss, legal trouble, or harm makes people act without thinking. | Phishing email threatens legal action unless a fine is paid. |

Tip: Authority and Fear can look alike. Authority is "obey me because of who I am." Fear is "act now or something bad happens to you." Urgency is about time pressure, Fear is about consequences.

### Why Social Engineering Works (3)
1. **Exploits natural tendencies**: wanting to help, trusting authority, acting fast under pressure.
2. **Circumvents technical controls**: even secure systems fall if the people using them are manipulated. That is why it is often the **entry point** for bigger attacks.
3. **Cognitive biases**:
   - **Confirmation bias**: favoring info that matches what you already believe.
   - **Availability bias**: overestimating how likely something is because of recent examples.

## Part 5: Zero Trust Model

- **Zero Trust**: a security model where **no one, inside or outside the network, is trusted by default**. Everyone must be verified before accessing resources, no matter where they are.

### Six Key Principles
1. **Never Trust, Always Verify**: threats can be outside or inside. Verify every request.
2. **Least Privilege Access**: give users and devices only the **minimum access** they need. Shrinks the attack surface.
3. **Micro-Segmentation**: split the network into small segments, each with its own access controls, to **limit lateral movement**.
4. **Identity and Access Management (IAM)**: strong authentication like **MFA** (multi-factor authentication, needing two or more proofs of identity).
5. **Continuous Monitoring**: watch user behavior and traffic for anomalies in real time.
6. **Data Protection**: encrypt sensitive data wherever it is, on-premises or cloud.

### Example Implementations (who used which three measures)

| Organization | Measures |
|---|---|
| Large commercial bank | User Authentication, Role-Based Access Control, Network Segmentation |
| Regional hospital | Patient Data Protection, Microsegmentation, Device Security |
| Software development firm | Continuous Monitoring, Secure Development Practices, API Security |
| Large e-commerce retailer | User Access Management, Data Encryption, Regular Audits |
| Government agency | Identity and Access Management (IAM), Assume Breach Approach, Policy Enforcement |

### Twelve Key Steps in Zero Trust Implementation
1. **Identify Resources and Data**: catalog all assets (data, apps, services). Rate each one's sensitivity and criticality.
2. **Define the Protection Surface**: protect the **most critical** data and assets, not the whole perimeter.
3. **Map the Transaction Flows**: understand how data moves between resources and users, and how users interact with resources.
4. **Implement Strong IAM**: MFA, plus **RBAC** (role-based access control, where access depends on your job role) and least privilege.
5. **Micro-Segmentation**: smaller segments with controls at each, to limit lateral movement.
6. **Continuous Monitoring and Analytics**: monitor user and entity behavior. Use **SIEM** to analyze logs and alerts.
7. **Automate Security Responses**: automation and orchestration tools for real-time response.
8. **Establish a Strong Endpoint Security Posture**: secure all devices including mobile and IoT. Use **EDR**.
9. **Regularly Test and Update Security Policies**: assessments, penetration testing, update policies for new threats.
10. **Educate and Train Users**: ongoing training, security-aware culture.
11. **Implement Data Encryption**: encrypt sensitive data **at rest and in transit**.
12. **Integrate Threat Intelligence**: use it to shape strategy and anticipate threats.

## Part 6: Malware

- **Malware**: malicious software or programs that perform malicious activities.
- Malicious activities shown on the slide: **Propagation, Theft, Destruction, Exploitation, Unauthorized Activities, Deception, Hidden Activities, Opening Backdoors**.

### Virus
- **Infects files.** **Infection** is a virus's ability to insert a copy of itself (or any malicious code) into another executable called the **host**.
- Viruses are executable code or programs.
- Some must be **loaded into memory first**, then infect files when triggered by events like executing, opening, or even closing a file.
- Acts like a biological virus attacking plants and animals.
- Infects by **injecting virus code into a host**.

#### Common Types of Viruses (by host)

| Type | Host it infects |
|---|---|
| **Boot sector virus** | **MBR** (Master Boot Record, the first sector the computer reads at startup) |
| **Binary file infector** | **PE and DOS executable files** (PE = Portable Executable, the Windows .exe/.dll format) |
| **Multipartite virus** | **Combination of boot and binary file** infectors |
| **Macro virus** | Office documents: **DOC, PPT, XLS**, etc. |
| **Script virus** | **HTML, VBS, BAT** |

#### Types of File Infections (from the diagrams)

| Infection type | Where the malware body goes | Result after infection |
|---|---|---|
| **Prepending** | **Before** the host | Malware body, then host body |
| **Appending** | **After** the host | Host body, then malware body |
| **Cavity** | Inside **empty gaps** in the host | Host, malware, host, malware, host (host size stays about the same) |
| **Overwriting** | **Overwrites** the start of the host | Malware body, then only the **remaining** host body (host is damaged) |
| **Sandwich (or Amoeba)** | At **both** the start and end | Malware, host, malware |
| **Complex** | At both ends, host is **compressed** | Malware, compressed host body, malware |

#### Operating Algorithm of Viruses
**Direct file infectors:**
1. Search directories and subdirectories for a target file that matches set criteria.
2. Check the target is suitable, then check if it was **already infected**.
3. If not, insert the malicious code and generate the payload (if any).
4. Pass control to the host or execute the host.

**Memory resident viruses:**
1. Install itself first. Do a **memory residency check** (is another copy already in memory?).
2. If not, allocate memory and load itself there to become **resident**.
3. **Hook DOS functions or APIs**. Hooking means intercepting calls to those functions. Any executable that uses the hooked function is checked for suitability and prior infection.
4. Insert the code, generate the payload, then pass control to the host or execute it.

Difference: a direct infector **goes looking** for files. A memory resident virus **sits in memory and waits** for programs to use the functions it hooked.

#### Mother Virus
- The **origin** of an infection. A program that searches for targets and copies its contents into them.
- Antivirus companies mark it with **"-O"** at the end of the detection name, for example **PE_VIRUS.A-O**.
- Its arrival looks like a trojan's, but it is **still classified as a virus** because of how its payload behaves.

#### Virus Payload
- **Payload**: the harmful action the malware carries out.
- Viruses were the **first malware to have payloads**. Not all malware have payloads.
- Also called **"symptoms of infection"** because most are visible to the user.
- Examples: display images or messages, generate sounds, run other apps abnormally, reboot or shut down automatically, slow down the computer, alter desktop and taskbar settings.

### Worms
- Malware that **copies itself and sends itself over the network** to infect other systems.
- Named after the tapeworm in **John Brunner's novel "The Shockwave Rider."**
- Replicates constantly **without needing another program** (no host file).
- Can keep replicating until it **fills memory, disk space, and network bandwidth**.
- Examples: **Code Red, Sircam, Nimda** ("admin" spelled backwards), **Klez**. **MyDoom** and **Netsky** are variants of multifaceted attack worms exploiting OS and app weaknesses.
- Can send itself to **all email addresses** on an infected system.
- Can infect **web servers** to target visitors.
- **How worms spread (7)**: Email, Internet Relay Chat (IRC), Instant Messaging apps, Peer-to-Peer file sharing, Network shares, Vulnerability exploits, Disk drives / thumb drives.
- **Characteristics**:
  - May act like viruses, backdoors, trojans, or keyloggers.
  - **NOT capable of infecting files**, but spreads itself across a network.
  - **No host required** (some argue the infected machine is the host).

### Trojans (Trojan Horses)
- **Cannot spread by themselves**. They have no way to send a copy of themselves to another computer.
- Get in by **riding on a trusted third-party application**.
- Named after the hollow wooden horse in Greek legend that hid Greek soldiers to get into Troy.
- **Hide their true nature** and show their real behavior only when activated.
- Often disguised as helpful software, like **readme.exe** bundled with shareware or freeware.
- Users are usually tricked by **social engineering** into running them.

#### Kinds of Trojans (9)

| Kind | Intention | Payload / key trait |
|---|---|---|
| **Destructive** | **To destroy** | Triggered by **time or a specific event**. Formats drives, overwrites MBR or boot sectors, deletes or corrupts files and data, deletes registry entries, disables hardware settings, hangs Windows, makes the OS unusable or unbootable. |
| **Dropper** | **Drop and execute** other malware | Acts as a **carrier**. Comes as an installer or setup package, or compressed files (WinZip, WinRAR). Often a by-product of apps that **join two or more executables**. Drops files, then runs them. **No internet needed.** |
| **Downloader** | **Download** malware and execute | Acts as an **agent**. **Needs an internet connection**. Uses **WININET APIs**. Usually downloads other trojans, sometimes worms and viruses. Connects to a malicious site, downloads, executes. |
| **Clicker** | Make users **visit malicious websites** | **Indirect**: redirects the user instead of downloading directly. Keeps trying to connect to the internet, modifies the **Windows HOSTS file**, changes browser home/search page, hijacks URLs typed in the address bar via auto-complete. |
| **Flooders / Nukers / DoS / DDoS** | Make computer **resources unavailable** | Modifies HOSTS file to block websites, **100% CPU usage**, makes network or internet hard to access. (Forms below.) |
| **Backdoor** | Let a remote attacker **connect to and control** the machine | Two parts: **server and client**. Opens/closes CD tray, pop-ups, runs apps, shuts down Windows, browses files, captures clipboard and screenshots. **Classified as a separate malware type** from trojans because of its behavior and damage potential. |
| **Spy** | **Monitor and log** user activity and send it to an unknown host. Trick users into revealing personal info. | Main focus: **information theft**. Sends logs via email, IRC, IM, FTP. May hijack browsing and intercept online banking or shopping. Steals accounts, passwords, **keystrokes**, system info, browsing info, **serial keys**, emails, clipboard, screenshots. |
| **Proxy** | **Install itself as a web proxy** on the machine | Two purposes: **sniff internet traffic** in the network, and **hide the hacker's true location**. Payload: accept all internet traffic. |
| **Ransomware** | **Encrypt user files** and extort money for restoring them | Primary motivation: **money**. Payload: encrypt user data files. |

#### Forms of DoS (under Flooders/Nukers)
- **DoS** (Denial of Service): general term for malware denying users their computing privileges.
- **Land attack**: a local area network denial attack. Redirects all packets the infected machine sends **back to itself**.
- **Ping flood (Ping of Death)**: uses **ICMP (ping) packets** to attack a target.
- **SYN flood**: uses **TCP SYN packets**. Normally every SYN gets an ACK reply. Too many SYNs at once and the target cannot reply anymore, so communication is denied.
- **Flooders**: flood the **network** with garbage packets.
- **Nuking**: sending lots of garbage packets to a **single target machine**.
- **DDoS** (Distributed DoS): **simultaneous nuking** of one target from **several sources**.

#### Backdoor Components
- **Server component**: spread across the internet, remotely controlled by the hacker through the client.
- **Client component**: held by the hacker, used for remote access.
- Note: a later slide (Other Forms of Attacks: Backdoors) describes it the other way, with the **client backdoor file on the victim** and the **server program with the remote malicious user**. If asked, follow the wording of the question. The Trojan Backdoor slide (server on victim, client with hacker) is the more detailed one.
- Two communication styles on the later slide:
  - **Server and Client Communication**: backdoor file on the victim, the other program with the attacker.
  - **Command and Control (C&C)**: the victim connects to a **compromised website** that the attacker uses to send malicious commands.
- **Backdoor vs Remote Access Tool (RAT)**: a backdoor is a trojan giving an attacker remote access. A **Remote Access Tool** is a **legitimate** admin tool for remote control and troubleshooting, for example **RDP / Windows Remote Desktop Connection**.

### Malware Infection Channels (7)
Removable media, Email, Network shares, Internet chat, Web, Peer-to-peer applications, Vulnerabilities and exploits.

### Malware Installation Techniques (3)
1. **Reside in memory**
2. **Drop copies of itself**
3. **Place auto-start mechanisms**

#### Memory Residency Techniques
- The **first step** in malware execution: loading itself into memory and staying there unnoticed.
- Methods (4): **normal background process**, **service process**, **injected DLL** (Dynamic Link Library), **injected thread** (runs code inside another process's address space).

#### Dropped Copies
- Part of installation: drop files to complete the infection.
- Types (3): **copies of itself**, **components**, **other malware**.

#### Auto-start Mechanisms
- Let malware run every time the system starts. **Anything that triggers malware to run** counts as an auto-start mechanism.
- **Startup folders**:
  - C:\Windows\Start Menu\Programs\Startup
  - C:\Windows\All Users\Start Menu\Programs\Startup
  - C:\Documents and Settings\<User_Name>\Start Menu\Programs\Startup
  - C:\Documents and Settings\All Users\Start Menu\Programs\Startup
- The startup folder path is stored in the **registry** under the **Shell Folders** and **User Shell Folders** keys (in both HKEY_CURRENT_USER and HKEY_LOCAL_MACHINE, ...\Microsoft\Windows\CurrentVersion\Explorer\). Values are "Startup" (current user) and "Common Startup" (all users).
- Changing that value to any other folder makes **every executable in the new folder run** at startup.
- **Win.ini** (older Win9x): a [windows] section with **load=malware.exe** and **run=malware.exe**.
- **Windows NT/XP and later**: registry key HKEY_CURRENT_USER\Software\Microsoft\Windows NT\CurrentVersion\Windows with **"run"** and **"load"** values.
- **C:\Windows\Winstart.bat**: acts like a normal batch file (copies and deletes files) and runs **every time Windows starts**.
- **Autoexec.bat**: "automatically executed batch file," which **DOS** runs at boot.

### Understanding Malware Forms (6)
Binary files, Encrypted files, Polymorphisms, Packed files, Macros, Scripts.

#### Binary Malware
- Compiled executables like **Win32 / PE EXE and DLL** files.
- Compiled in **assembly** = smaller. Compiled in high-level languages like **Borland Delphi and Visual Basic** = larger. Others use Borland C/C++, Visual C/C++, Microsoft .NET, Java.

#### Encryption and Polymorphism
- Plain malware is easy to reverse-engineer. Opened in a **hex editor**, its strings give it away.
- AV scanners use **heuristic technology** to spot code sequences common to malware.
- Malware **encrypts** itself to avoid reverse engineering, detection, and recognition.
- **Polymorphism (polymorphic encryption)**: the malware uses a **different encryption every time it copies itself**.
- Diagram: **encrypted-only malware** looks the **same** in every generation copy. **Polymorphic malware** looks **different** in every generation copy.

#### Heuristics
- **Heuristics**: the rules antivirus follows to find potentially malicious code.
- Heuristic analysis compares file code to previously found threats. Based on how much it matches, it estimates the chance of a threat and flags it.
- Techniques (3):
  - **Dynamic scanning**: runs the file in a **simulated environment** and watches its behavior.
  - **File analysis**: checks the file's intent, destination, and purpose.
  - **Multicriteria analysis (MCA)**: weighs the potential threat.

#### Packed Malware
- **Compression** used to avoid reverse engineering, detection, and recognition.
- Normal compressors (WinZip, WinRAR, IZARC) make an executable **lose** its executable state.
- Malware compression keeps the file **still executable**. This is called **packing**.
- Purpose: smaller files that **spread easily** and **avoid detection and analysis**.

#### Macro Malware
- **Macro**: a small program inside a document that automates tasks.
- In Microsoft Office (**OLE**, object linking and embedding, files), macros are **VBA** (Visual Basic for Applications) commands.
- Malware writers put malware in Office documents through macros.
- The **LOVELETTER worm ("I Love You" virus)** was written in **Visual Basic Script**.

#### Script Malware
- **Scripts**: executable code that is **interpreted, not compiled**.
- Examples (12): VBS, JScript (Microsoft), JavaScript (Sun), HTML, XML, PHP, PERL, Python, Ruby, IRC Script, Batch files (DOS/Windows), BASH Script (Unix).
- **Script encoding**: scripts are easy to read and change, so Microsoft made a way to hide the code. Works **only in VBS and JScript**.
- Script malware uses encoding and **script encryption** to avoid detection. The slide shows normal VBScript (readable) next to encoded VBScript (gibberish).

## Part 7: Other Forms of Attacks

### Malicious Code
- Includes viruses, worms, trojan horses, **active web scripts**, and **bots**.
- **Bot**: an automated program that runs certain commands when it gets a specific input. Used in trojans, logic bombs, backdoors, and spyware.

### Spyware
- **Spyware**: technology that gathers info about a person or organization **without their knowledge**.
- **Web bug**: a tiny graphic on a web page or email that collects info about whoever views it.
- **Tracking cookie**: placed on the computer to track activity across websites and build a **detailed profile**.
- **Adware**: software meant for **marketing** (pop-ups).

### Virus Hoax with a Real Virus
- An attack that sends a **virus hoax** (a fake virus warning) **with a real virus attached**.
- People share legitimate-looking messages more readily, so the real virus spreads even when users think they are helping.

### Password Attacks
- **Password crack**: trying to **reverse** a password. Used when an attacker gets a copy of the **SAM** (Security Account Manager) file, which holds **hashed** passwords. The attacker hashes guesses with the same algorithm and compares. A match means the password is cracked.
- **Brute force attack**: uses computing power to try **every possible combination**. If attackers narrow the target accounts, they can focus more resources on them (another reason to **change default passwords**).
- **Dictionary attack**: a **variation of brute force**. Picks specific accounts and uses a **list of common passwords** instead of random combinations. Defense: organizations use similar dictionaries to block weak passwords at reset, and **require numbers or special characters**.
- **Account lockout policy**: locks an account after too many failed attempts (listed as a countermeasure under password attacks).

### Spoofing
- Sending messages with a **forged source IP address** so they look like they come from a **trusted host**, to get unauthorized access.

### Man-in-the-Middle (MITM)
- Also called **TCP hijacking**. The attacker **monitors packets, modifies them, and puts them back** into the network.
- Uses **IP spoofing** to impersonate another party. Lets the attacker eavesdrop and change, delete, reroute, forge, or divert data.
- In the hacker attacks section: the attacker sits **between the remote server and the requesting computer**, pretends to be the other party, and intercepts data both ways.

### Scareware
- Scares victims by **faking critical infections**. Made popular by **FAKEAV**, which uses a fake antivirus interface.
- Also a **cybercrime**, since it extorts money (often bitcoin).

### Social Engineering (malware deck)
- **Social engineering**: using social skills to convince people to reveal access or credentials. It manipulates or deceives people into actions against their normal behavior.
- Attacker may pose as someone **higher up** (like a CIO) or as **new or current employees needing help**.
- **Advance-Fee Fraud (AFF)**, internationally called **4-1-9 fraud**: uses fictitious companies, for example the Nigerian National Petroleum Company.
- Malware deceives users with **realistic attachment names** or embedded links, and deceptive message content.

#### Social Engineering Email Lures (7)

| Lure | How it works |
|---|---|
| **Mail Delivery Error** | Imitates **non-deliverable receipts (NDR)**. Uses unfamiliarity with email feedback. User opens an attachment or clicks to "see the original message." Usually contains a **worm**. |
| **Account Information** | Targets **bank and credit card accounts**. Asks the user to click a link. Method used by **phishing websites**. |
| **Generic Conversation** | Conversational, friendly message. The **basic type** of social engineering. |
| **Virus or Patch Alerts** | Plays on **fear of malware**. Warns of a dangerous new virus and tells you to install the attached "patch." Written to look legitimate. |
| **Non-English Language** | Often found with **worms**. English speakers get curious about foreign-language messages. |
| **Physical Attraction** | Short message offering pictures (women, pornography, celebrities). The "picture" is often the malware attachment. |
| **Free Downloads** | Lures by **greed**. Brief message offering easy money, cracked software, free MP3s. Used to spread **worms**. |

### Polymorphic Threats
- **Polymorphic malware** constantly **changes its identifiable features** to evade detection. Viruses, worms, bots, trojans, and keyloggers can all be polymorphic.
- **Polymorphic viruses**: complex file infectors that make **modified versions of themselves** but keep the **same basic routines**. They encrypt their code with **different keys** each time.
- One of the biggest challenges in fighting viruses and worms, since they defeat **signature-based detection** (looking for a known pattern). They change size and file traits to avoid detection.

### Phishing
- Trying to get personal or financial info by **posing as a legitimate entity**.
- **Spear phishing**: a message that looks like it is from an employer, colleague, or other legitimate contact, sent to a **small group or one specific person**. Slide example: a Computer Science student at a Philippine university who stole credit card info.
- Three primary techniques:
  1. **URL manipulation**: an HTML message with an embedded hyperlink that leads somewhere fake.
  2. **Web site forgery**: a fake site (the slide notes "using two buttons").
  3. **Phone phishing**: pure social engineering, **pretexting** over the phone.

### Blended Threats
- **Bundles of malicious programs** combining functions of different malware (trojans, worms, backdoors).
- Complex behavior, **multiple payloads**, often hard to detect.
- Often use an **infection chain**: the visitor is diverted to a malicious URL, then tricked into downloading a malicious file.

### Spam, Mail Bombing, Sniffers
- **Spam**: unsolicited email.
- **Mail bombing**: a **DoS attack using email**. Exploits **SMTP** (Simple Mail Transfer Protocol) flaws by sending large emails with forged headers to poorly configured mail systems.
- **Sniffer**: a program or device that **monitors data on a network**. Can be used legitimately for network management or to steal info. On TCP/IP networks called **packet sniffers**. Risky because many users send data on the LAN in **clear text**, so it shows passwords and file contents.

### Pharming and Timing Attack
- **Pharming**: **redirecting legitimate web traffic to a fake site** to steal info. Uses trojans or worms to change the URL the user typed. May exploit DNS so the real host name points to the fake site's IP, called **DNS cache poisoning**.
- **Timing attack**: explores the contents of a **web browser cache** and stores a **malicious cookie** on the client's system.

### Defending Against Malware (6)
1. Keep antivirus up to date.
2. Run full system scans often.
3. Keep OS and applications up to date.
4. Use firewalls (hardware and application).
5. Be wary of messages with links or attachments.
6. Use strong passwords.

## Part 8: Online Scams (Philippine context)

### Behaviors That Contribute to Online Scams
- **Links in SMS**.
- **SIM card registration**: registered SIMs being sold for **P500 per SIM**. Example: a **POGO** (Philippine Offshore Gaming Operator) site with **25,000 phones** and **600 workers** (400 Filipinos, 200 others).
- **Sharing complete names, mobile numbers, and email addresses**.

### Four Strategies to Prevent/Avoid Scams

| Strategy | Meaning | Points on the slide |
|---|---|---|
| **"Magdamot ka!"** (be stingy) | Do not give things away. | Never share your **6-digit verification code** (OTP). **Anything that asks for money is a scam.** |
| **"Magduda"** (be suspicious) | Doubt everything. | You are not special. Watch for scare tactics. **90% of scams come from Facebook.** Online shopping with payment outside legitimate e-commerce sites. |
| **"Dedma!"** (ignore it) | Do not engage. | **Robocalls**: answering tells them your number is active, then you get bombed with SMS. |
| **"Isumbong!"** (report it) | Report scams. | **PNP** for local complaints. **NBI** for national-scope complaints (NBI had only 9 personnel for this). PNP time is eaten up by cyberlibel cases. Victims' info (numbers, emails, names) is needed, so use your true identity on social media. **1326** is the online hotline for reporting scams. Slide also notes GCash is 40% owned by a Chinese company. |

### Other Prevention Points
- **Cyberhygiene**: change password **once a month**, always use **two-factor authentication**.
- Watch out for **QR code scams**.
- **Investment scams**: a legit investment doubling your money (100%) in 3 years is already lucky. Check with the **SEC**. Real firms usually hold a second license too. Scammers usually hand out copies of their licenses.
- **Scare scams, tech support scams, love scams**. **Tinder**: about **70% scams**.

### Dating Scam Profiles and Red Flags

| Profile | Red flags |
|---|---|
| **Sad boi / sad gurl** | Sad life story, appeals to pity, borrows money, unmatches you right after. Make sure your match is a real person first. |
| **The Seducer** | Super attractive photo, shares and asks private info, starts sexual talk, asks for nudes (for **blackmail**). Nudes online cannot be removed. |
| **The Escort** | Ultra attractive picture. |
| **Slow Burn** | Plays the long game, harmless conversations at first. |
| **The Predator** | Mature men targeting young women. Sends money, asks for nudes, insists on meetups, asks for discretion. |

## Part 9: Forms of Cyberattacks and Hacking

### Cyberattack
- The **intentional exploitation** of computer systems and networked organizations.
- Uses **malicious code** to modify computer data, code, or logic, which can lead to compromise of crucial data.
- Also a platform for **cybercrimes** like theft of information and identity.

### Hacking
- Finding flaws in a network or system and using them to gain access. Example: a password cracking algorithm.
- Hacking is a **crime** punishable in most jurisdictions.
- **Ethical hacking**: hacking considered beneficial, done by professionals, government agencies, and accredited institutions to counter malicious hackers.

### Types of Hackers

| Type | Permission? | Goal | Key detail |
|---|---|---|---|
| **Black hat** | No | Personal or financial gain, sometimes espionage or fun | Writes malware. Wants financial info, login credentials, personal info. |
| **White hat** | **Yes, asks the owner first** (this makes it legal) | Find and fix vulnerabilities | Ethical hackers. Employees or contractors. Do **penetration tests**. Same methods as black hats. |
| **Grey hat** | No | Mixed | Finds vulnerabilities **without permission**, reports them to the owner, often **asks for payment** to fix. If the owner refuses, **posts the exploit online**. |

### Ethical Hacking
- Attempting to infiltrate and exploit a system to find weaknesses so it can be secured.
- An **approved** bypass of security. The owner **grants permission**. It is **legal, planned, and approved**, unlike malicious hacking.
- Main goal: find weak points before malicious hackers use them.

### Penetration Testing
- A way to check how **robust** an organization's security is. Professionals **play the attacker** to find flaws before criminals do.
- Key objective: **identify and report** vulnerabilities.
- A **legal, commissioned hack**. Organizations usually do a **vulnerability assessment first** to find potential weaknesses, then the pen test.

### Common Hacker Attacks

#### MAC Spoofing
- Unauthorized change or faking of a device's **MAC address** (the hardware ID of a network card) to pose as one of your devices and intercept communication.
- Four ways:
  1. A simple MAC address change.
  2. Creating a random MAC address.
  3. Using a different manufacturer's MAC address.
  4. Configuring a new MAC address while keeping the current manufacturer part, then activating it automatically.

#### Denial of Service (DoS)
- Server gets **flooded with more traffic than it can handle**, causing massive delays.
- **DDoS**: many computers attack at once. Hard to handle because the traffic comes from many places worldwide at the same time.

#### Rogue DHCP Server
- A DHCP server set up on the network by an **attacker or an unaware user**. DHCP is the service that hands out IP addresses and network settings automatically.
- Not under the network admin's control. **Accidental example**: a user plugs in a DHCP-capable modem.
- Attackers use them for **sniffing, reconnaissance, and MITM**.
- Both the real and rogue servers hand out settings like **default gateways, DNS servers, WINS servers, and IP addresses**.

#### Cross-Site Scripting (XSS)
- **Injecting malicious code into a website** so it runs in visitors' **browsers**.
- Common method: put the code in a **comment** or a script that runs automatically. Example: a JavaScript link in a blog comment.
- Damages the site's reputation by putting users' info at risk.

#### SQL Injection
- **SQL**: the language used to talk to databases.
- Targets servers that store critical data in SQL databases. Malicious code makes the **server reveal info it normally would not**.
- Worse if the server stores private customer data (usernames, credit card numbers).

#### Session Hijacking
- Your computer and a web server share a **unique session ID** that must stay private.
- An attacker who gets the session ID can **take over the session**.

#### Password Attack
- Passwords are the most common way to authenticate users.
- **Dictionary attack**: uses common passwords from a dictionary.
- **Brute-force attack**: tries passwords randomly, sometimes with logic (user's name, hobbies, job title).
- Countermeasure: **account lockout policy**.

## Comparison Tables (easily confused terms)

### Virus vs Worm vs Trojan vs Backdoor

| | Virus | Worm | Trojan | Backdoor |
|---|---|---|---|---|
| Needs a host file? | **Yes** | **No** | No (rides on a trusted app) | No |
| Spreads by itself? | Yes, by infecting files | **Yes, over the network** | **No** | No |
| Infects files? | **Yes** | **No** | No | No |
| How it gets in | Infected file runs | Email, IRC, IM, P2P, shares, exploits, USB | **Disguised as useful software**, social engineering | Like a trojan |
| Main trait | Has payloads ("symptoms") | Fills memory, disk, bandwidth | Hides true nature until activated | **Remote control** (server + client) |
| Examples | Boot, macro, script viruses | Code Red, Nimda, Klez, MyDoom | readme.exe | Classified separately from trojans |

### Trojan Dropper vs Downloader vs Clicker

| | Dropper | Downloader | Clicker |
|---|---|---|---|
| Role | **Carrier** | **Agent** | Bait |
| Internet needed? | No (malware is inside it) | **Yes** (WININET APIs) | Yes |
| How it delivers malware | Drops files it already carries, then runs them | **Directly** downloads and runs | **Indirectly**: redirects user to a malicious site |

### Encryption vs Polymorphism vs Packing vs Script Encoding

| Technique | What it does | Copies look the same? |
|---|---|---|
| Encryption | Scrambles the malware body to hide strings | Yes, every copy the same |
| Polymorphism | Different encryption each new copy | **No**, every copy different |
| Packing | Compresses but **stays executable** | Smaller file |
| Script encoding | Hides script code (VBS and JScript only) | Readable code becomes gibberish |

### Brute Force vs Dictionary vs Password Crack

| | Brute force | Dictionary | Password crack |
|---|---|---|---|
| Method | Every possible combination | **List of common passwords** on chosen accounts | Hash guesses and compare to the stolen **SAM** hashes |
| Speed | Slowest | Faster (narrower) | Offline, needs the SAM file |
| Defense | Strong passwords, change defaults, lockout | Block dictionary words at reset, require numbers and symbols | Protect the SAM file |

### Phishing vs Spear Phishing vs Pharming vs Spoofing

| | What it is | Target |
|---|---|---|
| Phishing | Posing as legit to get personal or financial info | Anyone, broad |
| Spear phishing | Phishing that looks like it is from a boss or colleague | **Small group or one person** |
| Pharming | **Redirects** real traffic to a fake site (DNS cache poisoning) | Anyone typing the real URL |
| Spoofing | **Forged source IP** to look like a trusted host | Systems that trust that host |

### MITM vs Session Hijacking vs Sniffing

| | What the attacker does |
|---|---|
| Sniffing | **Only watches** traffic (passive) |
| Session hijacking | **Steals the session ID** and takes over the session |
| MITM | Sits **in the middle**, reads **and changes** packets, uses IP spoofing |

### DoS Variants

| Attack | Key trait |
|---|---|
| Land attack | Infected machine's packets redirected **to itself** |
| Ping flood / Ping of Death | **ICMP** packets |
| SYN flood | Too many **TCP SYN** packets, no room to send ACK |
| Flooding | Garbage packets over the **network** |
| Nuking | Garbage packets at **one machine** |
| DDoS | Nuking one target from **many sources** |
| Mail bombing | DoS using **email** (SMTP flaws) |

### Red vs Blue vs White vs Purple
See the table in Part 3. One line each: Red attacks, Blue defends, White referees, Purple combines both for learning.

### Black vs White vs Grey Hat
See the table in Part 9. The one deciding factor is **permission**. White hats ask first. Grey hats do not ask but report (and want money).

### Ethical Hacking vs Penetration Testing vs Vulnerability Assessment

| | What it is |
|---|---|
| Vulnerability assessment | **Done first**. Finds potential weaknesses. |
| Penetration testing | A **legal, commissioned hack** that proves the weaknesses can be exploited. |
| Ethical hacking | The broader practice of approved hacking to find weaknesses. Pen testing is one form. |

### SIEM vs SOAR vs EDR vs XDR vs UEBA vs ASM
See the table in Part 1. Quick split: SIEM **collects and correlates**, SOAR **acts with playbooks**, EDR **guards endpoints**, XDR **unifies everything**, UEBA **watches behavior (insiders)**, ASM **maps exposed entry points**.

### Backdoor vs Remote Access Tool

| | Backdoor | Remote Access Tool |
|---|---|---|
| Legit? | No, malware | **Yes** |
| Used by | Attacker | Administrators |
| Example | Trojan backdoor | RDP, Remote Desktop Connection |

## Applied Context (Midterm Activity 1 and Midterm Project)

- **Activity 1 (ransomware scenario, email server down)**: the **Incident Manager** coordinates using the **printed contact list** (because email and chat may be down) and does no technical work. The **Tech Manager** handles the ransomware as subject matter expert and brings in experts with the IM's okay. First NIST step applied was **Containment** (short-term mitigation: pull infected machines off the network), then **Eradication** (remove the malware, check unaffected systems too).
- **Midterm Project ("One Password Away", IRP for SLU)**: shows how the slides apply to a school. Incidents included a grade sheet emailed to the wrong Google Group (a **confidentiality** breach and a Data Privacy Act issue) and a phishing case that could have hit **integrity** (changing grades). Recovery steps: new passwords with two-step verification, restore from clean backups, bring systems back in stages, confirm data with offices like the Registrar. Post incident review: one report with timeline and root cause, retrospective meeting, report to the National Privacy Commission when needed, and reuse the incident as the next **tabletop exercise**. AI section repeats the slide limits: false alarms, needs skilled tuning, legal and ethical calls stay with people.
- Essay tip: if asked to apply the IRP, walk through the **six NIST steps in order** and name **IM, TM, CM** with one concrete action each.

## Likely Exam Questions / Quick Self-Check

1. **What is incident response?** An organization's processes and technologies for detecting and responding to cyberthreats, breaches, or attacks.
2. **Define a security incident.** Any digital or physical breach that threatens the confidentiality, integrity, or availability of systems or data.
3. **List the 7 common security incidents.** Ransomware, phishing and social engineering, DDoS, supply chain attacks, insider threats, privilege escalation, MITM.
4. **What is a CSIRT?** The computer security incident response team, made of stakeholders across the organization, that creates and runs the IRP.
5. **What is an IRP?** A written document, approved by senior leadership, that guides the organization before, during, and after an incident.
6. **Why print the IRP and contact list?** Email, chat, and document storage may be down during an incident.
7. **What is a tabletop exercise (TTX)?** A roleplaying simulation where a facilitator presents a scenario and feeds updates to see how each person plays their role.
8. **What is a holding statement?** A pre-prepared press response for when a reporter asks about a breach.
9. **Why review the IRP quarterly?** The best IRPs are living documents that change with the business.
10. **Who leads the response but does no technical work?** The Incident Manager.
11. **What is time dilation and who handles it?** People losing track of time during a crisis. The IM watches the clock.
12. **Who is the subject matter expert?** The Tech Manager.
13. **Who talks to reporters and social media?** The Communications Manager.
14. **What happens after an incident?** Retrospective (postmortem) meeting, update policies, communicate findings to staff.
15. **Six parts of an IRP?** Playbook, security solutions, business continuity plan, IR methodology, communications plan, documentation instructions.
16. **Six NIST steps in order?** Preparation, Detection and Analysis, Containment, Eradication, Recovery, Post Incident Review.
17. **Which phase is continuous?** Preparation.
18. **Two categories of containment?** Short-term mitigation and long-term containment.
19. **Difference between eradication and recovery?** Eradication removes the threat. Recovery restores systems to normal.
20. **Goal of post incident review?** Find the root cause, how the attacker got in, fix vulnerabilities, gather lessons learned.
21. **Which tool fights alert fatigue?** SIEM (AI-driven automated triage also helps).
22. **Which tool is best for insider threats?** UEBA.
23. **Which tool uses playbooks?** SOAR.
24. **Three ways AI improves IR?** Faster anomaly detection, more proactive response, predicting likely attack channels.
25. **Three limitations of AI in IR?** False positives, complexity, need for human oversight.
26. **Red vs Blue vs White vs Purple?** Attackers, defenders, referees, combined learning team.
27. **Direct vs indirect attacks?** Direct goes after the secret or brute forces it. Indirect tricks people.
28. **Five red team responsibilities?** Determine objectives, exploit vulnerabilities, compromise security, evade detection, develop report.
29. **What is breakout time?** The window between compromising the first machine and moving laterally to others.
30. **1-10-60 rule?** Detect in 1 minute, assess in 10 minutes, eject in 60 minutes.
31. **Seven principles of social engineering?** Authority, urgency and scarcity, reciprocity, social proof, liking, commitment and consistency, fear and threats.
32. **Why is social engineering effective?** It exploits natural tendencies, gets around technical controls, and uses cognitive biases.
33. **Confirmation vs availability bias?** Confirmation favors what you already believe. Availability overrates likelihood based on recent examples.
34. **Core idea of Zero Trust?** Never trust, always verify. No one is trusted by default, inside or outside.
35. **Six Zero Trust principles?** Never trust always verify, least privilege, micro-segmentation, IAM, continuous monitoring, data protection.
36. **Why micro-segmentation?** To limit lateral movement.
37. **What is the protection surface?** The most critical data and assets, which Zero Trust protects first instead of the whole perimeter.
38. **What is a virus host?** The executable or file the virus inserts itself into.
39. **Five virus types by host?** Boot sector (MBR), binary file infector (PE/DOS), multipartite (both), macro (DOC/PPT/XLS), script (HTML/VBS/BAT).
40. **Six file infection types?** Prepending, appending, cavity, overwriting, sandwich (amoeba), complex.
41. **What is a memory residency check?** A memory resident virus checking if another copy of itself is already in memory.
42. **How is a mother virus labeled?** With "-O" at the end, like PE_VIRUS.A-O.
43. **Why are payloads called symptoms of infection?** Most are visible to the user.
44. **Main difference between a worm and a virus?** A worm needs no host and spreads over the network. A virus infects files.
45. **Why can't trojans spread by themselves?** They have no mechanism to send copies of themselves.
46. **Nine kinds of trojans?** Destructive, dropper, downloader, clicker, flooders/nukers/DoS/DDoS, backdoor, spy, proxy, ransomware.
47. **Why is a backdoor classified separately from trojans?** Its unique behavior and damage potential.
48. **Two purposes of a trojan proxy?** Sniff internet traffic and hide the hacker's location.
49. **Land attack vs SYN flood?** Land redirects packets back to the sender itself. SYN flood overloads the target with SYN packets so it cannot reply with ACK.
50. **Three malware installation techniques?** Reside in memory, drop copies, place auto-start mechanisms.
51. **Four memory residency methods?** Normal background process, service process, injected DLL, injected thread.
52. **Name auto-start mechanisms.** Startup folders, registry Shell Folders keys, Win.ini load/run, registry run/load in Windows NT, Winstart.bat, Autoexec.bat.
53. **Six malware forms?** Binary, encrypted, polymorphic, packed, macro, script.
54. **Encryption vs polymorphism?** Encrypted copies all look the same. Polymorphic copies use different encryption each time.
55. **What is packing?** Compression that keeps the file executable, making malware smaller and harder to detect.
56. **Three heuristic techniques?** Dynamic scanning, file analysis, multicriteria analysis.
57. **What was LOVELETTER written in?** Visual Basic Script.
58. **Which script types support script encoding?** VBS and JScript only.
59. **Web bug vs tracking cookie?** A web bug is a tiny graphic that collects info about the viewer. A tracking cookie tracks activity across sites to build a profile.
60. **Backdoor vs remote access tool?** Backdoor is malicious. RAT (like RDP) is a legitimate admin tool.
61. **What is scareware?** Malware that fakes critical infections (like FAKEAV) to extort money.
62. **Three phishing techniques?** URL manipulation, web site forgery, phone phishing.
63. **What is a blended threat?** A bundle combining several malware types, with multiple payloads and an infection chain.
64. **What is pharming?** Redirecting real web traffic to a fake site, possibly through DNS cache poisoning.
65. **What is mail bombing?** A DoS attack using large emails with forged headers that exploits SMTP flaws.
66. **Six ways to defend against malware?** Updated antivirus, frequent full scans, updated OS and apps, firewalls, caution with links and attachments, strong passwords.
67. **Four strategies against scams?** Magdamot ka, Magduda, Dedma, Isumbong.
68. **PNP vs NBI for scam complaints?** PNP for local, NBI for national scope.
69. **Scam reporting hotline?** 1326.
70. **Black vs white vs grey hat?** Black: no permission, for gain. White: has permission, legal. Grey: no permission, reports and asks for payment, may post the exploit.
71. **What comes before a pen test?** A vulnerability assessment.
72. **Four ways to spoof a MAC address?** Simple change, random address, another manufacturer's address, new address keeping the current manufacturer and activating it automatically.
73. **What is a rogue DHCP server?** A DHCP server set up by an attacker or unaware user, outside the admin's control.
74. **XSS vs SQL injection?** XSS injects code into a website that runs in users' browsers. SQL injection makes a database server reveal data it should not.
75. **What makes session hijacking possible?** Stealing the private session ID.

### Possible Essay Prompts (with what to hit)
- **"Explain the IR lifecycle using a ransomware scenario."** Six NIST steps in order. Preparation (training, TTX), detection (SIEM/EDR alerts on mass encryption), containment (isolate machines), eradication (remove malware, check all systems), recovery (restore from backups, patch), review (root cause, lessons learned).
- **"Why is social engineering so effective, and how do you defend against it?"** Three reasons from the slide plus examples from the seven principles. Defense: training (Blue Team education), Zero Trust verification, MFA.
- **"Compare Red, Blue, and Purple Teams and the benefits of the exercise."** Roles, breakout time, 1-10-60, five benefits.
- **"How would you implement Zero Trust in an organization?"** Pick 5 or 6 of the 12 steps and tie them to an example from the five organizations.
- **"Should AI replace human incident responders?"** No. AI speeds detection, triage, and reporting, but the slides say it should assist, with false positives and legal or ethical decisions needing humans.

## Practice MCQs

1. Which IR phase is described as both the first and a continuous one?
   A. Detection and Analysis  B. Preparation  C. Recovery  D. Post Incident Review
   **Answer: B**

2. During an incident, who leads the response but performs no technical duties?
   A. Tech Manager  B. Communications Manager  C. Incident Manager  D. CSIRT attorney
   **Answer: C**

3. Removing malware and booting a rogue user off the network happens in which phase?
   A. Containment  B. Eradication  C. Recovery  D. Detection and Analysis
   **Answer: B**

4. Which technology is most effective at identifying insider threats that mimic authorized traffic?
   A. ASM  B. SOAR  C. UEBA  D. EDR
   **Answer: C**

5. According to the 1-10-60 rule, an adversary should be ejected within:
   A. 1 minute  B. 10 minutes  C. 60 minutes  D. 24 hours
   **Answer: C**

6. Which team oversees a Red/Blue exercise and ensures rules are followed?
   A. Purple  B. White  C. Green  D. Blue
   **Answer: B**

7. An attacker says "many of your colleagues already installed this update." Which principle is used?
   A. Authority  B. Reciprocity  C. Social proof  D. Liking
   **Answer: C**

8. In Zero Trust, dividing the network into smaller segments mainly prevents:
   A. Phishing  B. Lateral movement  C. Password cracking  D. Spam
   **Answer: B**

9. A virus that infects both the MBR and executable files is a:
   A. Macro virus  B. Script virus  C. Multipartite virus  D. Boot sector virus
   **Answer: C**

10. In which file infection does the malware body end up on both sides of the host?
    A. Prepending  B. Appending  C. Cavity  D. Sandwich (Amoeba)
    **Answer: D**

11. Which malware cannot infect files but spreads by itself over a network?
    A. Virus  B. Worm  C. Trojan  D. Macro
    **Answer: B**

12. Which trojan needs an internet connection and uses WININET APIs to fetch malware?
    A. Dropper  B. Clicker  C. Downloader  D. Proxy
    **Answer: C**

13. Malware that uses a different encryption each time it copies itself is:
    A. Packed  B. Polymorphic  C. Encoded  D. Binary
    **Answer: B**

14. An attack that redirects users to a fake site by poisoning DNS is:
    A. Phishing  B. Spoofing  C. Pharming  D. Sniffing
    **Answer: C**

15. A hacker finds a flaw without permission, reports it, asks for payment, and posts the exploit if refused. This is a:
    A. White hat  B. Black hat  C. Grey hat  D. Red team member
    **Answer: C**

16. A DoS attack that sends a large number of TCP SYN packets so the target cannot answer with ACK is a:
    A. Land attack  B. Ping flood  C. SYN flood  D. Mail bombing
    **Answer: C**

17. Which of these is a legitimate tool, not malware?
    A. Backdoor  B. Remote Desktop Connection  C. FAKEAV  D. Trojan proxy
    **Answer: B**

18. A dictionary attack is a variation of:
    A. Spoofing  B. Brute force attack  C. Session hijacking  D. XSS
    **Answer: B**

## Slide Content Notes
- The older "Incident Response Plan and AI (1)" file is identical to Midterms 1. No extra content.
- Midterms 3 slides 5 to 7 (file infection types), 32 to 33 (registry auto-start entries), 38 (encryption vs polymorphism), and 43 (encoded VBScript) are images. They were read from the rendered slides and written out above.
- Midterms 3 slide 47 has no specific title, only "Other Forms of Attacks." The content is a virus hoax with a real virus attached, so it is labeled that way here.
- The two backdoor slides disagree on which side holds the server and which holds the client (noted under Backdoor Components).
