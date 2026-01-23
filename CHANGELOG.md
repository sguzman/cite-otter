# Changelog
## [Unreleased]

### 📚 Documentation
- Cover release readiness for v0.5.0 ([1bdcd9a](https://github.com/sguzman/cite-otter/commit/1bdcd9a33a12febca3a7468ba3a20ff537c02024))

### 🚀 Features
- **(training)**: Validate finder datasets ([36733cb](https://github.com/sguzman/cite-otter/commit/36733cbf614e96695605a652f81badf0947d8ebf))
- **(delta)**: Include finder datasets ([5520f4d](https://github.com/sguzman/cite-otter/commit/5520f4d25fbce397a480069dd66178fd905d840c))## [0.4.0] - 2026-01-23

### ⚙️ Miscellaneous
- Persist report ([3189c1d](https://github.com/sguzman/cite-otter/commit/3189c1de57327dc9db2a95410628afb41b652e97))
- Ignore deepwiki md from lychee and typos ([a208471](https://github.com/sguzman/cite-otter/commit/a20847179d5cf46e19d0bc0468c593649ac02cdf))

### 📚 Documentation
- Docs: point anystyle links to github ([439af42](https://github.com/sguzman/cite-otter/commit/439af425a389e959e11b8e22c8711d098a1303bd))
- Describe training report samples and moved docs ([b12d0cc](https://github.com/sguzman/cite-otter/commit/b12d0cc6e209c60cbc293f9ee64b620758693a85))
- Ignore deepwiki from typo check ([c58637b](https://github.com/sguzman/cite-otter/commit/c58637ba64a1196d2dd8e0903f355f65e5f53800))

### 🚀 Features
- **(parser)**: Structured author metadata ([23fa19f](https://github.com/sguzman/cite-otter/commit/23fa19ffce942212a4ebe4c2163a4398691b4fa9))
- **(cli)**: Support parse format flag ([981520e](https://github.com/sguzman/cite-otter/commit/981520e021a6c9ccfe69728b10f3ccc9c5c07e16))
- **(cli)**: Support parse format flag ([d1f4223](https://github.com/sguzman/cite-otter/commit/d1f42233049e4cb3f0a4804b4cfb37d0d9451e8f))
- **(finder)**: Persist training signatures ([0601a82](https://github.com/sguzman/cite-otter/commit/0601a82cb03658f00c8e0b705c3393bc2d46529d))
- **(training)**: Persist sequence model signatures ([9421181](https://github.com/sguzman/cite-otter/commit/9421181248021c93728777fe816af9d3f67e6b6a))
- **(learning)**: Init ([8e02f32](https://github.com/sguzman/cite-otter/commit/8e02f3244f5749c9e4d9b5859a7ebb890317f808))
- Parser heuristics and training/delta validation ([1b7022c](https://github.com/sguzman/cite-otter/commit/1b7022c37bfd16876b4c9dc83ea207af1a4f1594))
- **(parser)**: Extend metadata heuristics & validate training outputs ([9102ffd](https://github.com/sguzman/cite-otter/commit/9102ffd95181102007453513818a0c25bc403da1))
- **(format)**: Align outputs with AnyStyle metadata ([9333710](https://github.com/sguzman/cite-otter/commit/9333710f528a6454e27327a6f0f4ca17855bb1ef))
- **(format)**: Add journal normalizer & sample CLI output ([7add570](https://github.com/sguzman/cite-otter/commit/7add5701d7b7c2caec8bb811cea0d35d6e825e6f))
- **(cli)**: Persist sample parse outputs ([192bc10](https://github.com/sguzman/cite-otter/commit/192bc102a756376295bbd48a002a28b214022dfe))
- **(parser)**: Normalize author and year heuristics ([0ddbccf](https://github.com/sguzman/cite-otter/commit/0ddbccf0fc3fc4ca21d50aba8e6b5d98c2e1ed45))## [0.3.0] - 2026-01-23

### ⚙️ Miscellaneous
- (normalizer): handle repeaters in names ([6667838](https://github.com/sguzman/cite-otter/commit/666783856391e7e690b8d147a3f06355bf7e1c2f))
- **(fmt)**: Toml ([3ed693b](https://github.com/sguzman/cite-otter/commit/3ed693b14a9e2a5c7057964ba72dfa48bb2efb60))

### 🐛 Bug Fixes
- **(cli)**: Borrowing ([0bad60f](https://github.com/sguzman/cite-otter/commit/0bad60f32837c0297683428b4f32ebeef3122ef9))
- **(cli)**: Borrow shared model paths ([d7b5e90](https://github.com/sguzman/cite-otter/commit/d7b5e90ae0b7d262487d5186c7ea00efcaab7e57))

### 🚀 Features
- **(parser)**: Reference parser/document/finder specs now execute (dictionary, format, normalizer tests are still ignored until their modules are implemented). ([cd35485](https://github.com/sguzman/cite-otter/commit/cd35485145cd0bd0172385b301e7064ac441d37b))
- **(dictionary)**: Add adapters and lookup helpers ([eef9e07](https://github.com/sguzman/cite-otter/commit/eef9e07e995bac18e62cb7d5f86dfe440bfc17c9))
- **(dictionary)**: Enable test ([3754736](https://github.com/sguzman/cite-otter/commit/375473694c82438ae0bc5614e1be11d9ed090ee6))
- **(cli)**: Enhanced stubs ([039b476](https://github.com/sguzman/cite-otter/commit/039b476684b3ff0318debe4f9fcdd9c7c307c8de))
- **(cli)**: Prints ([1773708](https://github.com/sguzman/cite-otter/commit/1773708d5f46683a8d9604e05804971952877901))
- **(cli)**: Training checks ([786e5cc](https://github.com/sguzman/cite-otter/commit/786e5cc1a1bcd74b9fa87fb6e281ec9f9a08de7f))
- **(cli)**: Parsing ([54111ea](https://github.com/sguzman/cite-otter/commit/54111eadf90cd413d87d2a3c23f8afc2c098675d))
- **(parser)**: Align labels with reference context ([54b3c8b](https://github.com/sguzman/cite-otter/commit/54b3c8b2f0b9b28ef34b12fe25c09d9ef42f010a))
- **(parser)**: Enrich metadata heuristics & training reports. ([3130f3d](https://github.com/sguzman/cite-otter/commit/3130f3d92c84813469bab898a20a54df76f84837))
- Polish ([a4ec1f8](https://github.com/sguzman/cite-otter/commit/a4ec1f8eb66f0b3a557630c6ab3b6e6cd397b87b))## [0.2.1] - 2026-01-22

### 🐛 Bug Fixes
- **(semver)**: Broken semver ([0071d5f](https://github.com/sguzman/cite-otter/commit/0071d5f78047013da4470d5a8657aa226b6572f0))

### 🚀 Features
- **(core)**: Parse and cli ([7e58e22](https://github.com/sguzman/cite-otter/commit/7e58e22ea7b0e2a7a9476e7d25091f6cc2be66e3))
- **(parser)**: Imp ([4c8fddf](https://github.com/sguzman/cite-otter/commit/4c8fddfaa2a2d5c72da7d0fa727412cd3bccc3d8))
- **(parser)**: Extended ([31430d2](https://github.com/sguzman/cite-otter/commit/31430d20527483d1e575e9b100bb7a00b05adf61))
- **(parser)**: Prepare ([c8e9e1f](https://github.com/sguzman/cite-otter/commit/c8e9e1f4db0caf2e375394f2b070bfc35b7187e7))
- **(parser)**: Align tokens/metadata with reference suite ([181b427](https://github.com/sguzman/cite-otter/commit/181b42781cb5fdf900ed3fde48d73687764ee4f8))

### 🚜 Refactor
- **(fmt)**: Toml ([b9de022](https://github.com/sguzman/cite-otter/commit/b9de02232410228aeff780af5b8fb0518e6301ca))## [0.2.0] - 2026-01-21

### ⚙️ Miscellaneous
- Initial commit ([f3f577a](https://github.com/sguzman/cite-otter/commit/f3f577a6ff50e1a8bd48fb13f12abc9fbb6f22db))
- **(template)**: Init ([375aa29](https://github.com/sguzman/cite-otter/commit/375aa29973365cf4c3d4e65619b9babf9d65916a))

### 🐛 Bug Fixes
- **(docs)**: Badge color ([6e4c438](https://github.com/sguzman/cite-otter/commit/6e4c438a3f3de340937418a522817dfcaec23a74))
- **(docs)**: Badges ([e4456aa](https://github.com/sguzman/cite-otter/commit/e4456aa0f97b1c2070e1dbe9c9de71f498fc1012))
- **(clippy)**: Defaults ([3636ed7](https://github.com/sguzman/cite-otter/commit/3636ed79f32c44a2c866b4c839d65c6a60838e90))

### 📚 Documentation
- **(branding)**: Add 3 branding art ([82683fa](https://github.com/sguzman/cite-otter/commit/82683faca52d5af4a609d71429aca0774aad0f6f))
- Readme docs ([270e57a](https://github.com/sguzman/cite-otter/commit/270e57acc7869f582feca9979c0419f213dd6d03))
- Reference to project ([9aa4113](https://github.com/sguzman/cite-otter/commit/9aa41134b4ab57b25d190b2d95e5411b2b9ff2e9))
- **(roadmap)**: Init ([f46eb81](https://github.com/sguzman/cite-otter/commit/f46eb814d70009b559fe0625c4d68425b126cea9))
- **(readme)**: Cleaned ([32f5184](https://github.com/sguzman/cite-otter/commit/32f51843c4663992acf7cac1f6b77a4fbb623af6))
- **(branding)**: Macot desc and color ([38c85c3](https://github.com/sguzman/cite-otter/commit/38c85c37e3d38064d2842ab66787b3675b610e44))
- **(typo)**: Mascot.txt ([f66c66d](https://github.com/sguzman/cite-otter/commit/f66c66d3a6b828aa7d52e514bd1da4751d27870f))
- **(badges)**: Init ([247a9b8](https://github.com/sguzman/cite-otter/commit/247a9b84c7285fa075bee4ea766f6d5a0f7b5279))
- **(just)**: No workspace and more stats commands ([4cf8d7d](https://github.com/sguzman/cite-otter/commit/4cf8d7db43b643372c68ad115cb123a944727925))
- **(branding)**: More favicons ([f7b3f9f](https://github.com/sguzman/cite-otter/commit/f7b3f9fb7ed216bae7a4a27f1122ee1fd460f75c))

### 🚀 Features
- **(just)**: Add ci friendly commands ([6933c04](https://github.com/sguzman/cite-otter/commit/6933c040e69bcbb6dba32fdb4d02dd999e4f3626))
- **(just)**: Clean up ci pipeline ([40f2644](https://github.com/sguzman/cite-otter/commit/40f26444793538d9b062b2e8c6f9dd21cc377a80))
- **(just)**: Llvm-lines ([2292fc6](https://github.com/sguzman/cite-otter/commit/2292fc65a4704e65137e6a770b65a36573fec018))
- **(tests)**: Implement tests ([f4a020b](https://github.com/sguzman/cite-otter/commit/f4a020bc237ddce2f1978ece4161ea2389d6d2a0))
- **(test)**: Compilation ([fb2fb3d](https://github.com/sguzman/cite-otter/commit/fb2fb3d30c5903f62b9c858a3076eb9e5c8bfe6a))
- **(core)**: Impl ([bee7c52](https://github.com/sguzman/cite-otter/commit/bee7c52bace0f8ba4878bdfcc430edc0457ebb5c))

### 🚜 Refactor
- Refactor(fmt) ([0922b56](https://github.com/sguzman/cite-otter/commit/0922b5607ad6a7af154af270a144291fa72fad26))<!-- generated by git-cliff -->
