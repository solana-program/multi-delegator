import { GILL_EXTERNAL_MODULE_MAP } from 'gill';

export default {
  idl: './programs/multi_delegator/idl/multi_delegator.json',
  before: [
    './codama-visitors.mjs#addEventAuthorityPda',
    './codama-visitors.mjs#setEventAuthorityAndSelfProgramDefaults',
  ],
  scripts: {
    js: {
      from: '@codama/renderers-js',
      args: [
        './clients/typescript/src/generated',
        {
          dependencyMap: GILL_EXTERNAL_MODULE_MAP,
          dependencyVersions: {
            gill: '^0.14.0',
          },
        },
      ],
    },
    rust: {
      from: '@codama/renderers-rust',
      args: [
        './clients/rust/src/generated',
        {
          crateFolder: 'clients/rust',
          formatCode: true,
        },
      ],
    },
  },
};
