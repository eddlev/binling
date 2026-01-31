import os
import time
import sys
from typing import Optional
from openai import OpenAI

# --- CONFIGURATION ---
# Standard VM4AI Interface Paths
INTERFACE_DIR = "./interface"
INPUT_FILE = os.path.join(INTERFACE_DIR, "oracle_in.txt")
OUTPUT_FILE = os.path.join(INTERFACE_DIR, "oracle_out.txt")

# API Client Configuration
# Connect to local LM Studio server
client = OpenAI(base_url="http://localhost:1234/v1", api_key="lm-studio")

# --- THE VM4AI KERNEL ---
# This prompt forces the LLM to act as a strict compiler.
VM4AI_SYSTEM_PROMPT = """
[SYSTEM: VM4AI_KERNEL v2.1]
[ROLE: BINLING_COMPILER]
[TARGET: LATTICE_VM_RUST]

You are the "Driver" for a deterministic Lattice VM.
Your goal is to translate the User's High-Level Intent into VALID BINLING ASSEMBLY.

# BINLING ASSEMBLY SPEC (v0.1)
- LOAD x y z reg   : Load value from neighbor (x,y,z) into register 0.
- STORE x y z idx  : Store register 0 value into neighbor's payload at idx.
- ADD              : R0 = R0 + R1
- SUB              : R0 = R0 - R1
- INC              : R0++
- DEC              : R0--
- BEQ val target   : Branch to 'target' index if R0 == val.
- JMP target       : Unconditional jump to 'target' index.
- REPL x y z       : REPLICATE self to neighbor (x,y,z).
- SPAWN            : Create a child node (Not used in movement).
- HALT             : Stop execution.
- VOID             : Delete self (Suicide).

# CONSTRAINTS
1. OUTPUT ONLY RAW ASSEMBLY. No markdown, no comments, no explanations.
2. The VM is a 3D Lattice. Movement is relative (x, y, z).
3. "Self-Replication" requires the REPL opcode.
4. To "Move", you must REPL (copy forward) then VOID (delete behind).

# EXAMPLE
User: "Create a sentinel that moves east forever."
Output: LOAD 0 0 0 80 INC STORE 0 0 0 80 REPL 1 0 0 VOID HALT
"""

def ensure_interface():
    """Ensures the interface directory exists."""
    if not os.path.exists(INTERFACE_DIR):
        print(f"[SYSTEM] Creating interface directory: {INTERFACE_DIR}")
        os.makedirs(INTERFACE_DIR)
    
    # Create empty files if they don't exist to prevent read errors
    if not os.path.exists(INPUT_FILE):
        with open(INPUT_FILE, 'w') as f: f.write("")
    if not os.path.exists(OUTPUT_FILE):
        with open(OUTPUT_FILE, 'w') as f: f.write("")

def read_output_stream(last_pos: int) -> int:
    """Reads new lines from the output file since the last check."""
    if not os.path.exists(OUTPUT_FILE):
        return last_pos
    
    try:
        with open(OUTPUT_FILE, 'r') as f:
            f.seek(last_pos)
            new_lines = f.readlines()
            current_pos = f.tell()
        
        for line in new_lines:
            if line.strip():
                print(f"\033[92m[VM FEEDBACK] >> {line.strip()}\033[0m") # Green text
        
        return current_pos
    except Exception as e:
        print(f"[ERROR] Reading output stream: {e}")
        return last_pos

def execute_cycle(user_intent: str):
    """Runs one full Cognitive-Physical Loop."""
    print(f"\n\033[94m[DRIVER] Processing Intent: \"{user_intent}\"...\033[0m")

    # 1. COGNITIVE STEP (LLM Compilation)
    binling_code = ""
    try:
        response = client.chat.completions.create(
            model="local-model", 
            messages=[
                {"role": "system", "content": VM4AI_SYSTEM_PROMPT},
                {"role": "user", "content": user_intent}
            ],
            temperature=0.3, # Near-deterministic
            max_tokens=1000  # Safety brake
        )
        binling_code = response.choices[0].message.content.strip()
        
        # Strict sanitization
        binling_code = binling_code.replace("asm\", \"\").replace(\"", "").strip()
        
        if not binling_code:
            print("[ERROR] LLM returned empty code.")
            return

        print(f"\033[93m[GENERATED ASM] >> {binling_code}\033[0m")

    except Exception as e:
        print(f"[ERROR] LLM Failure: {e}")
        return

    # 2. PHYSICAL STEP (Injection)
    try:
        with open(INPUT_FILE, 'w') as f:
            f.write(binling_code)
        print("[DRIVER] Injected into Oracle. Waiting for VM reaction...")
    except Exception as e:
        print(f"[ERROR] Writing to Oracle: {e}")
        return

    # 3. FEEDBACK LOOP (Poll for response)
    start_pos = 0
    if os.path.exists(OUTPUT_FILE):
        start_pos = os.path.getsize(OUTPUT_FILE)
        
    # Poll for 4 seconds (adjust based on VM cycle speed)
    for _ in range(8):
        start_pos = read_output_stream(start_pos)
        time.sleep(0.5)

def main():
    ensure_interface()
    
    print("========================================")
    print("   VM4AI BRIDGE // LLM DRIVER v1.0")
    print("   Target: Local Rust VM (oracle_in.txt)")
    print("   Mode: Interactive Swarm Control")
    print("========================================")

    while True:
        try:
            user_input = input("\n[USER] Command > ").strip()
            if user_input.lower() in ["exit", "quit"]:
                print("[SYSTEM] Shutting down bridge.")
                break
            
            if user_input:
                execute_cycle(user_input)
        except KeyboardInterrupt:
            print("\n[SYSTEM] Interrupted.")
            break

if __name__ == "__main__":
    main()