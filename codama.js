import { createCodamaConfig } from 'gill';

export default createCodamaConfig({
  idl: './programs/multi_delegator/idl/multi_delegator.json',
  clientJs: './clients/typescript/src/generated',
  clientRust: './clients/rust/src/generated',
});
