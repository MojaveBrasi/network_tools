***ROADMAP***

# Near Term Goals
- [] Basic Tokio architecture
- [] Database schema for captures
- [] Database schema for cataloging Ip/Macs
- [] Settings.json (serde)
- [] Send & Handle ARP packets

## Longer Term Goals
* Catalogue and match Ip/Macs based on known standars
    - Link local address ranges
    - Broadcast/Gateway addresses
    - Static addresses on LAN, if any
    - Resolve addresses with DNS, attach addrs to names
* GUI Client
    - Should be optional. User can always run headless
    - Maybe something Like Tauri that can be a webpage or a desktop app.
    - Dashboard with stats
    - Simple and fast charts.
    - Scope of this is yet to be known. Avoid too much complexity
* Functionality beyond network traffic analysis
    - Who knows
