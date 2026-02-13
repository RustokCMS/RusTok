# Testing guidelines

These guidelines capture what we should take from the shared rules to keep tests reliable and fast.

## Risks tests can introduce
- **Flaky failures**: async races, timeouts/sleeps, test ordering, external dependencies (DB/ports). Prefer deterministic synchronization over `sleep`.  
- **Test-only design debt**: avoid `#[cfg(test)]` hacks that leak into production design (e.g., making fields public or adding globals just to test).  
- **Build/feature breakage**: test-only dependencies must align with workspace features and avoid system-only libs when possible.  
- **False confidence**: tests that mock away core behavior can pass while real flows break.  
- **Slow suites**: slow integration tests discourage execution and degrade CI feedback loops.  

## Layered approach
1. **Unit**: fast, deterministic, no DB/network/time dependencies. Focus on pure logic and DTO mappings.  
2. **Integration**: uses DB/services, but no external network. Validate migrations, repositories, and service wiring.  
3. **Contract/Golden**: a small set of end-to-end checks for the most critical business flows and API compatibility.  

## Async tests
- Avoid `sleep()` as a synchronization mechanism.  
- Prefer polling with timeouts (retry until state/event observed).  
- Assert on outcomes from event handlers and queues instead of timing assumptions.  

## Database isolation
- Use a transaction per test with rollback, or  
- Use unique tenant identifiers + cleanup strategy.  

## Mocking boundaries
- Mock **ports** (e.g., `PricingPort`, `InventoryPort`, `TaxPort`) when unit testing services.  
- Avoid mocking internal persistence layers (e.g., SeaORM models) unless the test explicitly targets that integration boundary.  

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
