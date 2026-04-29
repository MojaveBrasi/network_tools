1. Remove default_db_dir and default_db_name from State — access them via state.settings.db_dir instead, eliminating the redundancy and the clone
2. Make Settings fields pub (or add getters) so they're accessible from outside the module
3. Wrap State in Arc<tokio::sync::RwLock<State>> in main() for safe async sharing
4. Initialize state in main() before the match block (fixing the existing main.rs errors too)
Does that match what you had in mind, or would you prefer to keep the convenience fields on State?
