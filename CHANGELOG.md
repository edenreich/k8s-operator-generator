## [1.10.0](https://github.com/edenreich/kopgen/compare/v1.9.0...v1.10.0) (2024-11-19)

### Features

* **hooks:** Add pre-commit hook for linting, static analysis, and testing ([512a164](https://github.com/edenreich/kopgen/commit/512a1640f65c7fbb52bc699a3c458f62b896eb37))
* **tests:** Add snapshot testing for main template and reorganize test structure ([797adf8](https://github.com/edenreich/kopgen/commit/797adf82716882f285f6aa654f615c4ac6a51da8))

### Improvements

* Change shebang in bump-version.sh to /bin/sh for consistency ([59fda4a](https://github.com/edenreich/kopgen/commit/59fda4a33dacbe16a5ee8a1b0ed863506d582075))
* Change shebang in install-git-hooks.sh to /bin/sh and enable exit on error ([7d2d5c3](https://github.com/edenreich/kopgen/commit/7d2d5c32e0482461fb5fed4d33339eb635391f29))
* **cli:** Enhance run command to conditionally use TARGET_ARCH ([e283d1f](https://github.com/edenreich/kopgen/commit/e283d1f708e30c090cd4f39c9e478a85ed3bcd7b))
* **cli:** Update default value for include_tags and enhance CLI argument parsing ([212af07](https://github.com/edenreich/kopgen/commit/212af077b344f4af867b49c3b7c8c15d22619002))
* **devcontainer:** Add markdown-all-in-one extension to development container ([ef1db9f](https://github.com/edenreich/kopgen/commit/ef1db9feb47fdbad6262776465654eb241b1d765))
* Improve formatting in templates_operator_main.rs.jinja and corresponding snapshot ([c8e9af4](https://github.com/edenreich/kopgen/commit/c8e9af4e01cec2da99e5013600f620d8143001fd))
* Move bump-version script to scripts directory and update reference in .releaserc.yaml ([b5c8cd4](https://github.com/edenreich/kopgen/commit/b5c8cd4970719a0e578529c2dd165c5835549ee3))
* Move install.sh script into scripts directory ([85ac5c3](https://github.com/edenreich/kopgen/commit/85ac5c3fd06d0c2832137d4d942a4d67bcada36f))
* Rename test function for clarity in templates_operator_main.rs ([453b2fc](https://github.com/edenreich/kopgen/commit/453b2fc12bd1a26090d534359eb03030a4c1a9db))
* **tests:** Add cargo-insta for snapshot review and update Taskfile for testing ([c2df119](https://github.com/edenreich/kopgen/commit/c2df119ff42db42160e64abc2810456f40043018))
* **tests:** Remove duplicate test case in config loading tests ([4f82f7d](https://github.com/edenreich/kopgen/commit/4f82f7da2cbf4715431c8be425c56d41589da005))

### Bug Fixes

* **tests:** Load utils mod once for all unit tests ([7dc0b98](https://github.com/edenreich/kopgen/commit/7dc0b988c911f9129156a39c6525d1e972c947d6))

### CI

* Update action-gh-release to v2 in release workflow ([a5689ed](https://github.com/edenreich/kopgen/commit/a5689eddbac91cb5c4b4de1f1f046ed7c3b21e61))

### Documentation

* Add contributing section to README for improved guidance ([4e9f8b7](https://github.com/edenreich/kopgen/commit/4e9f8b747f4aeac1dde71e1bc7bb14404ca6cd47))
* Remove unnecessary Kopgen heading from README for clarity ([11b17fc](https://github.com/edenreich/kopgen/commit/11b17fca4ac773bcf41b4a951962461be9c4c0d4))

### Miscellaneous

* Add snapshot tests for templates_operator_type and corresponding modules ([160c6ab](https://github.com/edenreich/kopgen/commit/160c6ab921340f029fc909aaaf5af28ac4df8ce5))
* Add templates_operator_controller and corresponding snapshot tests ([6df9f29](https://github.com/edenreich/kopgen/commit/6df9f29e9deb913125875a1ece099d643c405df1))
* Add uninstall script for pre-commit git hooks ([4860e1f](https://github.com/edenreich/kopgen/commit/4860e1f50d16ba14bb29ebf5ff45f1d2057988e8))
* Comment out snapshot test in Taskfile.yaml ([93e8bd8](https://github.com/edenreich/kopgen/commit/93e8bd84ccc43eb9704498df5864c2cea59f5282))
* Make TARGET_ARCH environment variable optional ([cc34c6c](https://github.com/edenreich/kopgen/commit/cc34c6c42fb1154d21cfc645f7ccd4acfcaa902d))

## [1.9.0](https://github.com/edenreich/kopgen/compare/v1.8.0...v1.9.0) (2024-11-17)

### Features

* **ci:** Add linting job to CI workflow and define lint task in Taskfile ([a7017b3](https://github.com/edenreich/kopgen/commit/a7017b3eee81626bc5d911979ffa6350a9d83247))
* **refactor:** Relying now on the CLI to pass the default values if not specified by the user via flags or environment variables ([fe887ab](https://github.com/edenreich/kopgen/commit/fe887ab9772eac924d6381b157edf8b2f7c1d913))

### Improvements

* Add 'markdown-all-in-one' extension to devcontainer configuration ([dcfd7af](https://github.com/edenreich/kopgen/commit/dcfd7af97feaaadc67f00ac4217b006ede112244))
* **cli:** Change log level for configuration output from info to debug ([b9e6075](https://github.com/edenreich/kopgen/commit/b9e60756274d499859df482f6d798f91e1fc146d))
* **cli:** Use operator_name from config for project name in init command ([4396029](https://github.com/edenreich/kopgen/commit/43960294919289bfcfcd6b53fffab1a872447449))
* **tests:** Add serial execution to generate command tests for improved reliability ([5a6022b](https://github.com/edenreich/kopgen/commit/5a6022b2573d7c863ff8148e6737c5cbc7f2b35e))
* **tests:** Enhance test descriptions and simplify parameter handling in generate command tests ([b808225](https://github.com/edenreich/kopgen/commit/b808225b66de18ecfed94b14170d9bce0d46830f))
* **tests:** Remove unused import of `serde_yaml::Error` in utils module ([690eaa7](https://github.com/edenreich/kopgen/commit/690eaa7ea20899b183540ba7ee3a1dcc94161eec))
* **tests:** Set and clear RUST_LOG environment variable in init command tests ([6def5b3](https://github.com/edenreich/kopgen/commit/6def5b3d0a3cff9835fb8eeba224dfb203efd623))
* **tests:** Simplify `init` command tests by extracting common assertions into helper functions ([3a60952](https://github.com/edenreich/kopgen/commit/3a6095202963a74d45bd67bf20cf924b6fd33b8e))
* **tests:** Simplify environment variable setup for Kubernetes operator tests ([fa0547c](https://github.com/edenreich/kopgen/commit/fa0547c2a960f7af742040c73605e2829f9c1fef))
* **tests:** Update `init` command tests to return Result and handle AppError ([3f7b856](https://github.com/edenreich/kopgen/commit/3f7b8562d13ee19c68190a082b61be34c82b628f))
* **tests:** Update CLI config tests to return Result and improve error handling ([90ae6ab](https://github.com/edenreich/kopgen/commit/90ae6ab51120c518770d2f146596951d42e0b633))
* **tests:** Update read_temp_file to use ? operator for error handling ([a7c8ea1](https://github.com/edenreich/kopgen/commit/a7c8ea15ac3a21786e251ee90b35f5ba80ef4500))

### Bug Fixes

* Generated project devcontainer - update kopgen CLI installation method to use raw content script ([b3d0ddf](https://github.com/edenreich/kopgen/commit/b3d0ddf05333ee0329c3eae946ca7440867d609a))

### Documentation

* Update README template to include project name ([3a0232b](https://github.com/edenreich/kopgen/commit/3a0232b7f1b6ee0d4f3f6e9696a6bd413caa431d))

### Miscellaneous

* Add descriptive comments for CLI command tests in the `kopgen` module ([25d6a71](https://github.com/edenreich/kopgen/commit/25d6a71bb518aa3edc83f4c3dcad06d43874994b))
* Refactor tests, ecapsulate them into their own module ([7d03d8c](https://github.com/edenreich/kopgen/commit/7d03d8c13be7c6f71c84d8fe36da228a93a53a6c))

## [1.8.0](https://github.com/edenreich/kopgen/compare/v1.7.1...v1.8.0) (2024-11-16)

### Features

* Add shell installation script ([#17](https://github.com/edenreich/kopgen/issues/17)) ([eab552e](https://github.com/edenreich/kopgen/commit/eab552e994fcc2630608e3955b4bd266657d52a8))

### Documentation

* **fix:** Update installation script URL in README to use raw content ([ba41961](https://github.com/edenreich/kopgen/commit/ba41961f60c273c167f778f478b9398ba182cfb7))
* **fix:** Update references from "Kubernetes Operator Codegen" to "Kubernetes Operator Generator (kopgen)" in CLI and configuration files ([7ae907a](https://github.com/edenreich/kopgen/commit/7ae907a900057bfe0333fa723ddae908d6dcf780))

## [1.7.1](https://github.com/edenreich/kopgen/compare/v1.7.0...v1.7.1) (2024-11-15)

### Bug Fixes

* Update PATH in zshrc template to include cross-compilation toolchains ([fb20f50](https://github.com/edenreich/kopgen/commit/fb20f50202151103870e86c39a3bd4f3df45ff3a))

### Miscellaneous

* Add explicit mapping for conventional commits ([b8b161d](https://github.com/edenreich/kopgen/commit/b8b161d0bb077b32acf56c9b785e66e8c5e0265e))
* Update changelog ([5d305b1](https://github.com/edenreich/kopgen/commit/5d305b18f4983cc9fe18f131999cc8d2a1d705a4))

# [1.7.0](https://github.com/edenreich/kopgen/compare/v1.6.0...v1.7.0) (2024-11-15)


### Improvements

* Pass schemas by reference instead of cloning in generate functions ([3fd9cff](https://github.com/edenreich/kopgen/commit/3fd9cffd36aea5453bdd5584e735f76e0b571e54))
* Replace unwrap with ? for error handling in generate_lib and generate_crdgen_file functions ([e501512](https://github.com/edenreich/kopgen/commit/e501512aca06d8cd2a7ad2571f7c595507b78681))
* Update template file extensions from .jinja to .yaml and .rs for consistency ([e081d5a](https://github.com/edenreich/kopgen/commit/e081d5a5bc02e0fe2e5da56cd440538fb1dad885))
* Update template paths for consistency and organization ([2587a50](https://github.com/edenreich/kopgen/commit/2587a50d9db6a114dfb28a289bba082b8ee35965))
* Update Prettierrc file extension from .prettierrc to .prettierrc.yaml for consistency ([6684d81](https://github.com/edenreich/kopgen/commit/6684d81ac6ef69b8ba00e1498a328b203f509c33))
* Update generated file headers to indicate they should not be edited manually ([836762a](https://github.com/edenreich/kopgen/commit/836762ad86c96ffcf2ec1d955fc3036c538c79a7))

### Bug Fixes

* **tests:** Update .prettierrc file extension to .prettierrc.yaml ([84f397e](https://github.com/edenreich/kopgen/commit/84f397e5b20e29ab752f67c41e3780e1b1b93b69))
* Update path for devcontainer dependencies template in bump-version script ([639d38e](https://github.com/edenreich/kopgen/commit/639d38e6fa20e78e30a72cf3eae92d0ec3d61a6c))
* Update path for devcontainer dependencies template in release configuration ([cf3d3ec](https://github.com/edenreich/kopgen/commit/cf3d3ecfb7b844b441e3df2618f9b9e81ab0cb1f))
* Update run command in taskfile to include 'run' argument for operator execution ([2dcd31c](https://github.com/edenreich/kopgen/commit/2dcd31ccddd8bfa34cd243de76d2ecdf46a96ef3))


### Features

* Add install task for CLI with cargo installation command ([74c849e](https://github.com/edenreich/kopgen/commit/74c849ef735429b87ac9e4c47ae24b0360808083))
* Configure cross-compilation for Rust binaries with musl target ([b4bb987](https://github.com/edenreich/kopgen/commit/b4bb987f358ffc828d07b76ad861626b2e9677f1))
* Enhance CI workflow for cross-compilation with musl targets ([c360dcd](https://github.com/edenreich/kopgen/commit/c360dcd37c552f9b727bbbac8423dbc3ec00d5ac))
* Implement CLI for Kubernetes Operator with command handling and CRD installation option ([e4d6ca4](https://github.com/edenreich/kopgen/commit/e4d6ca4642ce900c5b0d18884e57eb0871ea8ceb))
# [1.6.0](https://github.com/edenreich/kopgen/compare/v1.5.0...v1.6.0) (2024-11-14)


### Bug Fixes

* Correct file path for controller in .openapi-generator-ignore ([27fcf9a](https://github.com/edenreich/kopgen/commit/27fcf9a59bf14408bc499068edb9899491e40fef))
* Correct typo in build task description for kopgen CLI ([fa190a6](https://github.com/edenreich/kopgen/commit/fa190a61a2fd0faadda096c7987e6c6b8ca25b0f))
* Remove verbose flag from cargo build command in Taskfile.yaml ([90351b4](https://github.com/edenreich/kopgen/commit/90351b487252a9044aabf36e87804f0d2a0a5d77))
* Replace error handling with expect for missing extensions validation ([b9b2a09](https://github.com/edenreich/kopgen/commit/b9b2a0936c1b423c863d3a01bf21f2e125520c82))
* Simplify test command by removing verbose flag ([878296c](https://github.com/edenreich/kopgen/commit/878296c7662f152abbf967b3c6933f4475d2c153))
* Update build task description and remove verbose flag from cargo build command for the generated Taskfile ([b99098d](https://github.com/edenreich/kopgen/commit/b99098d4bd1e8c9cd13b5d09b465981149c39313))
* Update CLI run command to correctly pass arguments ([12b0888](https://github.com/edenreich/kopgen/commit/12b088889960913d464bae862d2f461786c78477))


### Features

* Add MissingRequiredExtension error and implement validation for Kubernetes extensions ([c35af2a](https://github.com/edenreich/kopgen/commit/c35af2a02847baa6699b14f00489b1da09cbd293))
* Add project directory path argument to generate command and update related function calls ([6013b61](https://github.com/edenreich/kopgen/commit/6013b6193afd378899d2255ce4f5faf35fb4d9ef))
* Introduce error handling with AppError and refactor related functions ([cfd8de7](https://github.com/edenreich/kopgen/commit/cfd8de7411ee4e9cbe91183d82ff09b78ddf7c94))

# [1.5.0](https://github.com/edenreich/kopgen/compare/v1.4.1...v1.5.0) (2024-11-12)


### Features

* Enhance Kubernetes operator generation with support for types and controllers automatically added in the main file ([1ba164d](https://github.com/edenreich/kopgen/commit/1ba164d7dab823dc2b63b2c677121385aea944db))
* Update cross-compilation support and dependencies for Rust binaries ([d4ceeff](https://github.com/edenreich/kopgen/commit/d4ceeffa892ce8b6da58976e3e5e781ecb606af5))

## [1.4.1](https://github.com/edenreich/kopgen/compare/v1.4.0...v1.4.1) (2024-11-01)


### Bug Fixes

* **security:** Release workflow was using a Github Action with a deprecated NodeJS version ([44835e0](https://github.com/edenreich/kopgen/commit/44835e0b77df43f7821b426a3a47aec3f462144d))

# [1.4.0](https://github.com/edenreich/kopgen/compare/v1.3.1...v1.4.0) (2024-11-01)


### Bug Fixes

* **hydrate:** Update command to use npx for prettier execution ([f2c1d6a](https://github.com/edenreich/kopgen/commit/f2c1d6aeaac4d6d7d4bde249b2e71adb724b91d1))
* **task:** Set TARGET_ARCH environment variable for release tasks ([8101c49](https://github.com/edenreich/kopgen/commit/8101c494f603d918ac3bd83107374b923a60ae17))


### Features

* **config:** Introduce configuration management and update environment variable handling ([2e933d2](https://github.com/edenreich/kopgen/commit/2e933d23d3c5ba46146094ebf3c4f7083f8c7421))

## [1.3.1](https://github.com/edenreich/kopgen/compare/v1.3.0...v1.3.1) (2024-10-31)

### Bug Fixes

- Update release configuration and bump version to v1.3.0 in cli.rs ([e0549c7](https://github.com/edenreich/kopgen/commit/e0549c7eab52c6a8e10c430602e78b79e6c13eb6))

# [1.3.0](https://github.com/edenreich/kopgen/compare/v1.2.0...v1.3.0) (2024-10-31)

### Features

- Enhance devcontainer setup with architecture-specific installations and additional tools ([6f241d5](https://github.com/edenreich/kopgen/commit/6f241d5df48b61d035a346342df2f4a532611a48))

# [1.2.0](https://github.com/edenreich/kopgen/compare/v1.1.0...v1.2.0) (2024-10-31)

### Features

- Make it easy to release a new version ([1668cf9](https://github.com/edenreich/kopgen/commit/1668cf9aa0f925a9cf69b520183d67adcefc0ebb))

# [1.1.0](https://github.com/edenreich/kopgen/compare/v1.0.1...v1.1.0) (2024-10-31)

### Features

- Add pull request template for improved contribution guidelines ([4dc2be7](https://github.com/edenreich/kopgen/commit/4dc2be766a12d654aa5aefe1d5128caaf3eaf1d4))
- Add release configuration for automated changelog generation ([fad849e](https://github.com/edenreich/kopgen/commit/fad849e536ab38e82c8eae536d74966044a8f367))
