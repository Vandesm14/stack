# TODO

Things that need done to get `origin/clean-up` at parity with `origin/main`

- [x] **Let**
  - [x] Consolidate into the scope system if we can
- [ ] **Tests**
  - [ ] Reimplement all or most tests for
    - [ ] Intrinsics
    - [x] Evaluation (scopes, etc)
- [x] **Compare**
  - [x] New behavior with the old, using the documentation as a reference for what **should** happen
    - **Note:** Lists get evaluated if they're not lazied (when pushed to the stack by the user)
- [ ] **Reimplement**
  - [x] **Intrinsics**
    - [x] All old intrinsics: List, String, etc
  - [x] **Imports**
    - [x] Either use namespaces or just get the old import behavior working
    - [x] Keep a list of sources based on runtime imports
  - [x] **Error Reporting**
    - [x] Bring back the errors that pointed to sources
