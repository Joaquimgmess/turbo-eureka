# Passive Tree and Combat Juice Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a Path of Exile-inspired constellation passive tree and explosive combat feedback.

**Architecture:** 
- Modular `PassiveNode` data structure stored in a global `PassiveTree` resource.
- `PlayerPassives` component to track unlocked nodes and available points.
- 'Star Map' UI using Bevy UI nodes and simple geometric sprites for connections.
- Event-driven combat triggers (OnHit, OnDeath) to execute passive effects like Ricochet and Shatter.

**Tech Stack:** 
- Bevy 0.14
- Bevy UI for the Star Map

---

### Task 1: Data Structures and Resources

**Files:**
- Modify: `src/components.rs`
- Modify: `src/resources.rs`

**Step 1: Define Passive Tree structures in components.rs**
Add `PassiveNode`, `PassiveEffect`, and update `PlayerPassives`.

**Step 2: Initialize global PassiveTree in resources.rs**
Create a resource to hold the tree topology.

---

### Task 2: Points and UI Trigger

**Files:**
- Modify: `src/systems/world.rs`
- Modify: `src/systems/ui.rs`
- Modify: `src/main.rs`

**Step 1: Update leveling to grant points**
Modify `collect_xp` to increment `available_points` in `PlayerPassives`.

**Step 2: Implement 'P' key to toggle Passive Tree UI**
Create a system to handle the 'P' key and a placeholder UI container.

---

### Task 3: Star Map UI Implementation

**Files:**
- Create: `src/systems/passive_ui.rs`
- Modify: `src/systems/mod.rs`

**Step 1: Build the Constellation UI**
Render nodes as circles and connections as lines. Handle clicking nodes to spend points.

---

### Task 4: Combat Triggers and Juice

**Files:**
- Modify: `src/systems/combat.rs`
- Modify: `src/systems/enemy.rs`

**Step 1: Implement Knockback**
Add physics-based pushback to `process_damage`.

**Step 2: Implement OnDeath Explosions**
Add check for "Explosion" passives in `check_enemy_death`.

---

### Task 5: Advanced Effects (Ricochet)

**Files:**
- Modify: `src/systems/combat.rs`

**Step 1: Implement Ricochet (Chain)**
Modify `update_projectiles` to seek new targets if the player has the Ricochet passive.
