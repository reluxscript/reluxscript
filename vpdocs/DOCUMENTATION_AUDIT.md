# Documentation Audit: Real vs Hallucinated Features

## Summary

This audit identifies features documented in `docs-mvp/` that don't actually exist in the codebase, likely from AI hallucinations in previous sessions.

---

## ✅ Core Hooks (Actually Implemented)

Based on `src/client-runtime/src/hooks.ts`:

1. **useState** ✅ Real (hooks.ts:68)
2. **useEffect** ✅ Real (hooks.ts:185)
3. **useRef** ✅ Real (hooks.ts:240)
4. **useServerTask** ✅ Real (hooks.ts:477)

---

## ✅ Task Scheduling Hooks (Actually Implemented)

Based on `src/client-runtime/src/task-scheduling.ts`:

1. **useMicroTask** ✅ Real (task-scheduling.ts:12)
2. **useMacroTask** ✅ Real (task-scheduling.ts:27)
3. **useAnimationFrame** ✅ Real (task-scheduling.ts:42)
4. **useIdleCallback** ✅ Real (task-scheduling.ts:59)

---

## ✅ Pub/Sub Hooks (Actually Implemented)

Based on `src/client-runtime/src/pub-sub.ts`:

1. **usePub** ✅ Real (pub-sub.ts:173)
2. **useSub** ✅ Real (pub-sub.ts:188)

---

## ✅ Pagination Hook (Actually Implemented)

Based on `src/client-runtime/src/usePaginatedServerTask.ts`:

1. **usePaginatedServerTask** ✅ Real (usePaginatedServerTask.ts:101)

---

## ✅ SignalR Hook (Actually Implemented)

Based on `src/client-runtime/src/signalr-hook.ts`:

1. **useSignalR** ✅ Real (signalr-hook.ts:22)

---

## ✅ Extension Hooks (Actually Implemented)

Based on `src/minimact-*/src/integration.ts`:

1. **useDomElementState** ✅ Real (minimact-punch)
2. **useDomQuery** ✅ Real (minimact-query)
3. **useDynamicState** ✅ Real (minimact-dynamic)
4. **useArea** ✅ Real (minimact-spatial)
5. **useDecisionTree** ✅ Real (minimact-trees)

---

## ❌ Hallucinated Features (NOT Implemented)

### 1. useClientState ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/guide/getting-started.md` (lines 299-322)
- `docs-mvp/v1.0/guide/concepts.md` (lines 39-47)
- `docs-mvp/v1.0/api/hooks.md` (lines 119-145)

**Claims:**
- "Client-only reactive state that never syncs to the server"
- "Perfect for UI state (search query, open/closed, hover)"

**Reality:** Does not exist in codebase. No file `useClientState.ts` or export in hooks.ts.

**Issue:** Conceptually flawed for dehydrationist architecture (can't work without client VDOM).

**Action:** REMOVE all references.

---

### 2. usePredictHint ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/api/hooks.md` (lines 148-172)

**Claims:**
- "Explicitly tell the prediction system about upcoming state changes"
- "For 100% cache hit rates"

**Reality:** Does not exist in codebase. No file `usePredictHint.ts` or export.

**Action:** REMOVE or mark as "Planned Feature" (if you actually want to implement it).

---

### 3. useMarkdown ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/api/hooks.md` (lines 175-193)

**Claims:**
- "Server-side markdown parsing and rendering"
- "Supports GitHub-flavored markdown"

**Reality:** Does not exist in codebase. No file `useMarkdown.ts` or export.

**Note:** There IS a `MarkdownHelper` in C# (Minimact.AspNetCore), but no client-side hook.

**Action:** Either:
- REMOVE from client hooks docs
- Document as C# helper instead (in server-side API docs)
- Implement if you want it

---

### 4. useTemplate ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/api/hooks.md` (lines 196-218)

**Claims:**
- "Apply layout templates to components"
- Available layouts: DefaultLayout, SidebarLayout, AuthLayout, AdminLayout

**Reality:** Does not exist in codebase. No file `useTemplate.ts` or export.

**Note:** There ARE layout components in C#, but no hook to apply them.

**Action:** REMOVE (layouts are applied via C# attributes or routing, not hooks).

---

### 5. useMemo ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/guide/concepts.md` (line 79)

**Claims:**
- Listed as a supported hook

**Reality:** Does not exist. Not in hooks.ts.

**Action:** REMOVE from "supported hooks" list (or implement if needed).

---

### 6. useCallback ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/guide/concepts.md` (line 80)

**Claims:**
- Listed as a supported hook

**Reality:** Does not exist. Not in hooks.ts.

**Action:** REMOVE from "supported hooks" list (or implement if needed).

---

### 7. useContext ❌ DOES NOT EXIST

**Documented in:**
- `docs-mvp/v1.0/guide/concepts.md` (line 81)

**Claims:**
- Listed as a supported hook

**Reality:** Does not exist. Not in hooks.ts.

**Action:** REMOVE from "supported hooks" list (or implement if needed).

---

## Other Documentation Issues

### UpdateClientComputedState (Exists but Misleading)

**Documented in:**
- `features_complete.md` mentions "Client-computed state integration"
- Code exists in `MiniactHub.cs`

**Reality:**
- The C# method exists
- But there's no client-side hook that calls it
- It's unclear how users would actually use this

**Action:** Either:
- Create a client hook that uses it (e.g., `useComputedState`)
- Document it as a low-level API for advanced users
- Remove from feature list if not user-facing

---

## Recommendations

### Immediate Actions (High Priority)

1. **Remove useClientState** from all docs
   - `getting-started.md` (section 299-322)
   - `concepts.md` (section 39-47)
   - `hooks.md` (section 119-145)

2. **Remove usePredictHint** from hooks.md
   - Or mark as "Planned" if you want to implement

3. **Remove useMarkdown** from hooks.md
   - Or move to C# API docs if MarkdownHelper should be documented

4. **Remove useTemplate** from hooks.md
   - Layout usage should be documented differently

5. **Fix "Supported Hooks" list** in concepts.md
   - Remove: useMemo, useCallback, useContext
   - Add: usePub, useSub, useSignalR, task scheduling hooks

### Create Accurate Hook List

**Core Hooks:**
- useState
- useEffect
- useRef

**Server Task Hooks:**
- useServerTask
- usePaginatedServerTask

**Pub/Sub Hooks:**
- usePub
- useSub

**SignalR Hook:**
- useSignalR

**Task Scheduling Hooks:**
- useMicroTask
- useMacroTask
- useAnimationFrame
- useIdleCallback

**Extension Hooks:**
- useDomElementState (minimact-punch)
- useDomQuery (minimact-query)
- useDynamicState (minimact-dynamic)
- useArea (minimact-spatial)
- useDecisionTree (minimact-trees)

---

## Future Considerations

### Hooks You Might Want to Implement

1. **useMemo** - Memoize expensive computations
2. **useCallback** - Memoize callbacks
3. **useComputedState** - For UpdateClientComputedState integration
4. **usePredictHint** - Explicit prediction hints (if valuable)

### Hooks That Don't Make Sense

1. **useClientState** - Conceptually flawed (no client VDOM)
2. **useContext** - Unclear how it would work server-side
3. **useTemplate** - Layouts are C# concern, not hook concern

---

## Search & Replace Guide

Use these searches to find and remove hallucinated content:

```bash
# Find all useClientState references
grep -r "useClientState" docs-mvp/

# Find all usePredictHint references
grep -r "usePredictHint" docs-mvp/

# Find all useMarkdown references
grep -r "useMarkdown" docs-mvp/

# Find all useTemplate references
grep -r "useTemplate" docs-mvp/

# Find all useMemo references
grep -r "useMemo" docs-mvp/

# Find all useCallback references
grep -r "useCallback" docs-mvp/

# Find all useContext references
grep -r "useContext" docs-mvp/
```

---

## Conclusion

**Real Features:** 15 hooks actually exist and work
**Hallucinated Features:** 7 documented hooks don't exist

**Impact:** Users following the docs will try to use hooks that don't exist, leading to confusion and frustration.

**Priority:** Clean up documentation BEFORE public release.

---

## Files to Update

1. `docs-mvp/v1.0/guide/getting-started.md`
   - Remove useClientState section (lines 299-322)
   - Update examples to use only real hooks

2. `docs-mvp/v1.0/guide/concepts.md`
   - Remove useClientState section (lines 39-47)
   - Fix "Supported Hooks" list (lines 76-84)

3. `docs-mvp/v1.0/api/hooks.md`
   - Remove: useClientState (119-145)
   - Remove: usePredictHint (148-172)
   - Remove: useMarkdown (175-193)
   - Remove: useTemplate (196-218)
   - Add: usePub, useSub, useSignalR, task scheduling hooks

4. `docs-mvp/DOCUMENTATION_IMPROVEMENTS.md`
   - Update to reflect accurate feature list
