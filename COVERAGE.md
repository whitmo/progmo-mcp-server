# Coverage Goals

## Current Status
- Target: 75% minimum coverage (enforced in CI)
- Current: Run `cargo tarpaulin --workspace` to check
- Goal: 100% coverage

## Exemptions
None currently - all code should be tested.
Coverage must be maintained or improved with each PR.

## Strategy
1. Pure Function Extraction
   - Move complex logic into pure functions
   - Keep effects (IO, network, etc) separate
   - Write exhaustive tests for pure logic

2. Testing Hierarchy
   - Unit tests for pure functions
   - Integration tests for effects
   - Property tests for complex algorithms

3. Coverage Improvement Plan
   - Phase 1: Hit 75% coverage with pure function tests
   - Phase 2: Add integration tests for effects
   - Phase 3: Add property tests
   - Phase 4: Reach 100% coverage

## Notes
- Do not comment out unused code - either remove it or test it
- All new code must include tests
- Coverage checked in CI pipeline
