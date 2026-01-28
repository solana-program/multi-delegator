import { createCodamaConfig } from 'gill';

export default createCodamaConfig({
  idl: '../programs/multi_delegator/idl/multi_delegator.json',
  clientJs: './src/generated',
});
