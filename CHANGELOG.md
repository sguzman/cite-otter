# Changelog
## [Unreleased]

### ⚙️ Miscellaneous
- **(dictionary)**: Add import CLI and backend inserts ([90ed491](https://github.com/sguzman/cite-otter/commit/90ed49131a702c8949b7ab8e12356d1cea1d5b4a))
- Authors ([2234e23](https://github.com/sguzman/cite-otter/commit/2234e23329014f63262c7c8c1c22b10f76ecea28))
- **(taplo)**: Ignore  dirs ([38a7399](https://github.com/sguzman/cite-otter/commit/38a73994eef4ccf427c3ef172d36c64c6b8df99e))
- **(ignore)**: Rumdl ([b64710a](https://github.com/sguzman/cite-otter/commit/b64710ae550bd7077776e485f3410ef92658c9de))
- **(rumdl)**: Integrating rules ([f83b026](https://github.com/sguzman/cite-otter/commit/f83b026284ca56014708a60808362276a887ab13))
- **(rumdl)**: More rules - almost done ([1382f1c](https://github.com/sguzman/cite-otter/commit/1382f1cbee02a44adeca6fe86f80fb55ec08d4f0))
- **(rumdl)**: Finished rumdl formatting options ([c962696](https://github.com/sguzman/cite-otter/commit/c9626967ee6e1d0d1018db5c0fa1c96d0b101115))
- **(tooling)**: Add rumdl to justfile and reference docs ([3cfda41](https://github.com/sguzman/cite-otter/commit/3cfda4132bd915b5663d00b91d149c1bd68bad54))
- **(fix)**: Just all ([7aea39a](https://github.com/sguzman/cite-otter/commit/7aea39af2ef1d7284173de9a3eee3fc34367475c))
- **(clippy)**: Resolve lint warnings across parser and tooling ([6d655be](https://github.com/sguzman/cite-otter/commit/6d655beab556e7a42f869f8e2fdfb604f7ea7641))
- **(parser)**: Split parser into domain modules ([dca6428](https://github.com/sguzman/cite-otter/commit/dca6428f5119163134165b66d680372e6248e189))
- Fix parser/format parity blockers (gdbm compile, DOI/URL extraction, pages labeling, and test/report isolation) ([695db29](https://github.com/sguzman/cite-otter/commit/695db29a3e49ccb4f5fee5ae82a3fb0cdb51ddf4))
- Add structured Ruby parity summaries and Hyperfine markdown reporting ([136694a](https://github.com/sguzman/cite-otter/commit/136694ae22efc84ed5e4d1a8b11aa91548f03ef7))
- **(workflow)**: Add fast/full post-change verification profiles and make training benchmarks opt-in ([59340c3](https://github.com/sguzman/cite-otter/commit/59340c31433ea09820aaecb2ee4901ac8298d989))

### 🐛 Bug Fixes
- **(just)**: Ignore tmp ([fb299c9](https://github.com/sguzman/cite-otter/commit/fb299c9999777acf3b788b8dec8f6a480b8d0527))
- **(parser)**: Preserve journal segments and citation author lists ([8d8106a](https://github.com/sguzman/cite-otter/commit/8d8106a232677856a591a5e772e1fd7047f6d603))
- **(rumdl)**: Ignore migration docs ([a115c4b](https://github.com/sguzman/cite-otter/commit/a115c4b123231fadd83912fb1e43206e37e2e7d0))
- **(rumdl)**: Valid options ([4c5d263](https://github.com/sguzman/cite-otter/commit/4c5d2639ffa1838c23ac421abc53710817ade7c4))
- **(rumdl)**: Sentence per line is not viable ([6c35348](https://github.com/sguzman/cite-otter/commit/6c3534802cdb598e8d0c5dcc50e21444a00ed5f0))
- **(readme)**: Typo ([fadc63d](https://github.com/sguzman/cite-otter/commit/fadc63ddc1889d831dd953b240b0e9801d8102c3))
- **(bench)**: Correct parse/find flags and allow CLI overrides ([448fcd7](https://github.com/sguzman/cite-otter/commit/448fcd7b30a9ca9614fd2c8a78f53dee6dd9b18c))

### 📚 Documentation
- **(structure)**: Clarify docs layout and repo directories ([06c86a9](https://github.com/sguzman/cite-otter/commit/06c86a96448a24c6421916947b089bff7245c1c7))
- **(structure)**: Clarify project-specific docs folder ([ba4946d](https://github.com/sguzman/cite-otter/commit/ba4946d18db66dc6ecb80599e1a6003aed5a5b1a))

### 🚀 Features
- **(cli)**: Add model/report overrides and dataset flags ([f2d312e](https://github.com/sguzman/cite-otter/commit/f2d312eed69cbdc4b08e28db06b9d9595a49132f))
- **(cli)**: Add dataset flags and check output parity ([125f1a4](https://github.com/sguzman/cite-otter/commit/125f1a474330340b17dfcae24274de46479029b7))
- **(dictionary)**: Add lmdb/redis backends and CLI lookup ([61150d1](https://github.com/sguzman/cite-otter/commit/61150d176d1f822b5b7878cdd3c14736759c4042))
- **(language)**: Add language and script detection ([36b6b43](https://github.com/sguzman/cite-otter/commit/36b6b439a58a46773b9027c980968f4c00c19c78))
- **(dictionary)**: Gate gdbm backend behind optional feature ([b3204e9](https://github.com/sguzman/cite-otter/commit/b3204e97b8d1f88a54b9055dd1c399f37b1975f1))
- **(dictionary)**: Add import CLI and backend inserts ([1a01a70](https://github.com/sguzman/cite-otter/commit/1a01a7025de0346c4f5f08e603869c567200d668))
- **(dictionary)**: Add AnyStyle import format and tag bitmasks ([d6d0cae](https://github.com/sguzman/cite-otter/commit/d6d0cae36dc755aa992611e4bcb40c3f6b270409))
- **(cli)**: Add dictionary sync for AnyStyle data ([8ddf7dd](https://github.com/sguzman/cite-otter/commit/8ddf7ddbd09c85826957a64f3b384d096200a01d))
- **(parser)**: Use dictionary tags for labeling and type resolution ([4f6f6e2](https://github.com/sguzman/cite-otter/commit/4f6f6e2d0901bf4cdbe28218513e8adc25e057c3))
- **(normalizer)**: Add abbreviation map and journal expansion ([fd21b18](https://github.com/sguzman/cite-otter/commit/fd21b18741afb5f7a4f04a59619710556c43729a))
- **(normalizer)**: Add normalization config pipeline ([2770b43](https://github.com/sguzman/cite-otter/commit/2770b43d1dc391b4c0e1593ee5eec3d04acac767))
- **(cli)**: Support normalization-dir for formatted output ([68ea580](https://github.com/sguzman/cite-otter/commit/68ea580d30f16390f7fc891cc11ebc94e388c880))
- **(cli)**: Add normalization sync command ([1a0207a](https://github.com/sguzman/cite-otter/commit/1a0207a2dff99a1f78c657873c302e35ec40fffa))
- **(parser)**: Apply normalization without losing field types ([88b083f](https://github.com/sguzman/cite-otter/commit/88b083f534e8bc3fe50fc1b7eac891d9cf68fa13))
- **(cli)**: Apply normalization in parse output ([d863b8c](https://github.com/sguzman/cite-otter/commit/d863b8c049677fc1e8325b7d0e9049327c54cc2e))
- **(normalizer)**: Add normalization-dir fixtures and tests ([dd04ad9](https://github.com/sguzman/cite-otter/commit/dd04ad922713734a1e93216cd3f85422e19fef89))
- **(cli)**: Allow normalization-sync from repo path ([db52dc3](https://github.com/sguzman/cite-otter/commit/db52dc3e8458318198c1e8f43a69144a35cbb986))
- **(cli)**: Document normalization sync and test repo clone ([2d72d0f](https://github.com/sguzman/cite-otter/commit/2d72d0f4f762e2c5459d4681850eb1ecd28ad3d9))
- **(cli)**: Support repo subdir for normalization sync ([166b33d](https://github.com/sguzman/cite-otter/commit/166b33d7efca4bb948c92fed3d69620df3c104df))
- **(parser)**: Improve author parsing and metadata coverage ([1486d12](https://github.com/sguzman/cite-otter/commit/1486d126069a344df28e9122b6b62d47882047aa))
- **(parser)**: Improve author parsing and metadata coverage ([ca32efc](https://github.com/sguzman/cite-otter/commit/ca32efc7dff80ada7a533cb51234e4810b8ecf29))
- **(format)**: Add CSL issued/page/volume/issue mapping ([ba3502d](https://github.com/sguzman/cite-otter/commit/ba3502d5f8d38c26f92e4ec82daecc44481da529))
- **(parser)**: Improve page ranges and date part parsing ([a904a27](https://github.com/sguzman/cite-otter/commit/a904a277a958647096dcc9dce6dcec86c33838eb))
- **(format)**: Emit CSL name objects for contributors ([d3f6871](https://github.com/sguzman/cite-otter/commit/d3f6871df2c12d55a5a3dd6267ea5aa62e5fce0a))
- **(parser)**: Parse month-name dates with trailing years ([054fcce](https://github.com/sguzman/cite-otter/commit/054fcced781b7d5a928aa040b24ef89f8cdd1b16))
- **(parser)**: Handle en dash year and month ranges ([f492ead](https://github.com/sguzman/cite-otter/commit/f492eadc6a40cce3d61067b8724d9ba794685f74))
- **(format)**: Emit CSL page-first for page ranges ([d13668b](https://github.com/sguzman/cite-otter/commit/d13668b5833c215c0482fb4e6e2643061aa75d8a))
- **(format)**: Include isbn/issn in BibTeX output ([2935209](https://github.com/sguzman/cite-otter/commit/2935209b6d1b0c96ce861e0e8659d907beae4ca0))
- **(parity)**: Expand parser/date edge cases and report validation ([47c20e4](https://github.com/sguzman/cite-otter/commit/47c20e49d56e7913167b765338d3ecf228cc8f49))
- **(format)**: Expand CSL fields and BibTeX issue mapping ([f1b82d9](https://github.com/sguzman/cite-otter/commit/f1b82d9525e83c859ec06b197df54515a57b8786))
- **(normalizer)**: Add locale overrides and parser/format parity tweaks ([962a787](https://github.com/sguzman/cite-otter/commit/962a7873eda78ecda5aaeb02670247672d225636))
- **(normalizer)**: Add locale overrides and parser/format parity tweaks ([1a52215](https://github.com/sguzman/cite-otter/commit/1a522155e08394feace00ca8d7c951c36d5b3296))
- **(format)**: Add parity snapshots and tighten parser/date handling ([1b54808](https://github.com/sguzman/cite-otter/commit/1b54808f562326be809b65685c54290d4ab6220f))
- **(parser)**: Expand edge-case parsing and core format snapshots ([5d3f4c8](https://github.com/sguzman/cite-otter/commit/5d3f4c809c2fa45bfed0507888e3c309f813d215))
- **(format)**: Expand core snapshot coverage and parsing fixtures ([ac30bd2](https://github.com/sguzman/cite-otter/commit/ac30bd225dde95db8ad4d2391a103ad315aa2364))
- **(test)**: Add snapshot diff helper and normalization sync script ([2d6e2aa](https://github.com/sguzman/cite-otter/commit/2d6e2aa1ff4b2be869fbf6cd19714d4f5fc3f134))
- **(test)**: Add core snapshot refresh and diff artifacts ([84a9f7a](https://github.com/sguzman/cite-otter/commit/84a9f7ae7a2ce9756aa2b97be1a7bfab77a3c507))
- **(test)**: Improve core serialization and snapshot diff reporting ([1352720](https://github.com/sguzman/cite-otter/commit/135272093c1b006cb1c27e969813b4c2127f8cdd))
- **(test)**: Refine core serialization and diff headers ([e4dba0b](https://github.com/sguzman/cite-otter/commit/e4dba0b1867c49f9c0205f78d6aa2ed8a746650b))
- **(test)**: Generate real normalization fixtures and improve diff headers ([d0d2587](https://github.com/sguzman/cite-otter/commit/d0d258718abea32e7b636e1e2231ad8ab219da3a))
- **(fixtures)**: Tag-aware serialization and snapshot summary headers ([2ad13c5](https://github.com/sguzman/cite-otter/commit/2ad13c5df331cc6c7fb26fbe8bdbdf1b3e6dbd0c))
- **(fixtures)**: Harden core reference rendering ([81c0d99](https://github.com/sguzman/cite-otter/commit/81c0d990a2196254c406041f97ef6e7e206ceea5))
- **(parity)**: Extend parser cases and report stats ([e6fbea2](https://github.com/sguzman/cite-otter/commit/e6fbea25c171a05aa0b8b2f75928b8a96c2e6c9d))
- **(reports)**: Add summary metrics to training and validation ([dc75e0f](https://github.com/sguzman/cite-otter/commit/dc75e0fe3644c5ef7ade02edfc6bbc3f06a450d3))
- **(reports)**: Add token deltas to delta report ([45c9bf8](https://github.com/sguzman/cite-otter/commit/45c9bf8cdc2b5a2976b7df7abfda6abc48ede196))
- **(format)**: Align CSL/BibTeX output with Ruby ([967d1cf](https://github.com/sguzman/cite-otter/commit/967d1cf457bd531e6bb212464d18a4a47154482b))
- **(parser)**: Tighten date extraction for core refs ([97902ec](https://github.com/sguzman/cite-otter/commit/97902ecd72a5cd3fd31e519977fdd7ae29ab5a15))
- **(parser)**: Improve author/title/date heuristics ([39d5814](https://github.com/sguzman/cite-otter/commit/39d5814068cad666581704e67cd3aa3678fc9fbd))
- **(parser)**: Capture citation numbers and improve author/date edge cases ([bcf9a7b](https://github.com/sguzman/cite-otter/commit/bcf9a7b0d5e124bd4ca1be05a87e876ec4ef4c42))
- **(parser)**: Improve journal/type inference and volume/issue parsing ([4f3236d](https://github.com/sguzman/cite-otter/commit/4f3236db3c8b2db442e6e474d4283f11f3c17269))
- **(parser)**: Improve title selection and page range extraction ([bb701e4](https://github.com/sguzman/cite-otter/commit/bb701e4705d8c048c860de95e2adb97aedb48f65))
- **(parser)**: Improve title selection, date parsing, and container cleanup ([48f6298](https://github.com/sguzman/cite-otter/commit/48f6298cd162caf13f447410a36e12829382acf4))
- **(parser)**: Improve author/title heuristics and page range formatting ([4701063](https://github.com/sguzman/cite-otter/commit/4701063a47c8197481fe831681433a2e5f9fb93c))
- **(parity)**: Track finder tokens and improve author parsing ([a54b766](https://github.com/sguzman/cite-otter/commit/a54b766bb1d91fa32128b235c1a37d659e953f69))
- **(parser)**: Add chapter/editor extraction and normalization asset checks ([13bbe40](https://github.com/sguzman/cite-otter/commit/13bbe40413f45716f556a07174ce31aeffa3912a))
- **(parser)**: Iterate author/title extraction heuristics ([12448d4](https://github.com/sguzman/cite-otter/commit/12448d45f674e38080e97a010461d0f47ea6cf56))
- **(parser)**: Refine author/journal parsing and volume/page extraction ([b979dac](https://github.com/sguzman/cite-otter/commit/b979dac5c222d9160cc7e98d4f730c73d74e2445))
- **(rumdl)**: Init ([95121d6](https://github.com/sguzman/cite-otter/commit/95121d63a2296c30d4a75dda63c77097ec308783))
- **(bench)**: Add hyperfine parity benchmark for ruby CLI ([e7ca418](https://github.com/sguzman/cite-otter/commit/e7ca418082c4112d380f9e69e5777156af73fa2d))
- **(bench)**: Add rust baseline and training parity hyperfine runs ([bca7d2c](https://github.com/sguzman/cite-otter/commit/bca7d2c6eedb0f9a161be1ea85a414750275cd67))

### 🚜 Refactor
- Fmt ([831a713](https://github.com/sguzman/cite-otter/commit/831a713df68ebdef7006c2b7d0840e8d66f8ee8c))
- Fmt ([e16566c](https://github.com/sguzman/cite-otter/commit/e16566c29edf70577bd8fa44cb6bb9439005e893))
- **(fmt)**: Add txt ([a25b34e](https://github.com/sguzman/cite-otter/commit/a25b34ea33a8fd9318b884d88f2386d06dcd887b))
- **(just)**: Add sh ([1d3501c](https://github.com/sguzman/cite-otter/commit/1d3501c2fd80b2460a7799011152eea541515869))
- **(fmt)**: Rumdl on markdown ([c8ad8a6](https://github.com/sguzman/cite-otter/commit/c8ad8a6a8447c4aa148683d7ade2f44da5e54d5a))
- **(parser)**: Split parser into core, types, and field token modules ([d40b8ad](https://github.com/sguzman/cite-otter/commit/d40b8ad3d4cb9fd96571057c2f1c62b58fb835d1))
- **(parser)**: Modularize extract helpers and exports ([14f1c38](https://github.com/sguzman/cite-otter/commit/14f1c3883f4a4f535d51e0edb3cddadbfd9a5d0f))

### 🧪 Testing
- **(cli)**: Assert delta report kind and counts ([39af1fb](https://github.com/sguzman/cite-otter/commit/39af1fb0a7210436095aa612a775ec080f35eb20))
- **(cli)**: Document validation/delta behavior without models ([60cc090](https://github.com/sguzman/cite-otter/commit/60cc09001c45ab182f17e8016a35cb6f5bac70db))
- **(cli)**: Cover invalid glob failure modes ([6cf2034](https://github.com/sguzman/cite-otter/commit/6cf2034dac5e2b7264d83c54567da2a3afb661cf))
- **(cli)**: Cover missing parser dataset behavior ([aa5e2a0](https://github.com/sguzman/cite-otter/commit/aa5e2a003147c2ae5f2cb3a177b131c1590525fe))
- **(cli)**: Cover unreadable dataset failures ([4d667b2](https://github.com/sguzman/cite-otter/commit/4d667b27201d84752dee711666e9bb401226b3b6))
- **(cli)**: Assert report schema keys ([4a4350a](https://github.com/sguzman/cite-otter/commit/4a4350ae85d675fecdd9c80bad85aa0b38762538))
- **(cli)**: Assert report entry schemas ([0ee7a06](https://github.com/sguzman/cite-otter/commit/0ee7a06180a141da3e139c953a3b6f4096958661))## [0.5.0] - 2026-01-23

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
