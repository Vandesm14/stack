# TODO

Things that need done to get `origin/clean-up` at parity with `origin/main`

- [x] **Let**
  - [x] Consolidate into the scope system if we can
- [ ] **Tests**
  - [ ] Reimplement all or most tests for
    - [ ] Intrinsics
    - [ ] Evaluation (scopes, etc)
    - [ ] E2E (test some of the old std lib)
- [ ] **Compare**
  - [ ] New behavior with the old, using the documentation as a reference for what **should** happen
- [ ] **Reimplement**
  - [ ] **Intrinsics**
    - [ ] All old intrinsics: List, String, etc
  - [ ] **Imports**
    - [ ] Either use namespaces or just get the old import behavior working
    - [ ] Keep a list of sources based on runtime imports
  - [ ] **Error Reporting**
    - [ ] Bring back the errors that pointed to sources