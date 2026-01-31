# VM4AI Prompting Guide

The VM4AI system is not a chatbot. It is a **Physics Engine**. 
The LLM converts your natural language into **BinLing Assembly (BASM)**. 
Therefore, your prompts must map to physical operations that the Lattice VM can actually perform.

## ✅ Supported Capabilities (What works)
The VM physics engine supports the following primitives. Your prompts should combine these logic blocks:

### 1. Spatial Replication
* **Keywords:** "Grow", "Replicate", "Build", "Extend".
* **Concept:** Creating new nodes at relative coordinates $(x,y,z)$.
* **Example:** *"Create a line of 10 nodes extending in the X direction."*

### 2. State Memory
* **Keywords:** "Remember", "Store", "Save", "Register".
* **Concept:** Storing integer values in the node's 4 registers (R0-R3).
* **Example:** *"Store the value 50 in register 0."*

### 3. Logic & Math
* **Keywords:** "Add", "Subtract", "Increment", "Calculate".
* **Concept:** Performing arithmetic on registers.
* **Example:** *"Increment register 0 by 1 every time you replicate."*

### 4. Branching (Decision Making)
* **Keywords:** "If", "Check", "Compare", "Decide".
* **Concept:** Changing behavior based on register values or coordinates.
* **Example:** *"If x is greater than 10, stop replicating. Otherwise, continue."*

---

## ⛔ Unsupported Prompts (What fails)
Asking for abstract or non-physical tasks will result in hallucinated assembly or compilation errors.

* ❌ **Abstract:** "Analyze the sentiment of this text." (No text processing ops exist).
* ❌ **External:** "Send an email to admin." (No network I/O ops exist).
* ❌ **Undefined:** "Create a beautiful pattern." (Too vague; specify the geometry).

## 💡 Pro-Tip: "Think in Geometry"
The best prompts describe **Shapes** and **Behaviors**.
* *Bad:* "Make a virus."
* *Good:* "Create a self-replicating probe that doubles its value in R0 every step and branches left if R0 > 100."