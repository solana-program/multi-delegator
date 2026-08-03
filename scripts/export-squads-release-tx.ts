import {
    ComputeBudgetProgram,
    PublicKey,
    SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    Transaction,
    TransactionInstruction,
} from '@solana/web3.js';
import bs58 from 'bs58';

const BPF_UPGRADEABLE_LOADER = new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111');
const MAX_TRANSACTION_BYTES = 1232;

function requireEnv(name: string): string {
    const value = process.env[name];
    if (!value) {
        throw new Error(`Missing required env var ${name}`);
    }
    return value;
}

function decodeTransaction(encoded: string): Transaction {
    return Transaction.from(bs58.decode(encoded.trim()));
}

function withoutComputeBudget(instructions: TransactionInstruction[]): TransactionInstruction[] {
    return instructions.filter(ix => !ix.programId.equals(ComputeBudgetProgram.programId));
}

function upgradeInstruction(
    programId: PublicKey,
    buffer: PublicKey,
    authority: PublicKey,
    spill: PublicKey,
): TransactionInstruction {
    const [programData] = PublicKey.findProgramAddressSync([programId.toBuffer()], BPF_UPGRADEABLE_LOADER);
    return new TransactionInstruction({
        data: Buffer.from([3, 0, 0, 0]),
        keys: [
            { isSigner: false, isWritable: true, pubkey: programData },
            { isSigner: false, isWritable: true, pubkey: programId },
            { isSigner: false, isWritable: true, pubkey: buffer },
            { isSigner: false, isWritable: true, pubkey: spill },
            { isSigner: false, isWritable: false, pubkey: SYSVAR_RENT_PUBKEY },
            { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
            { isSigner: true, isWritable: false, pubkey: authority },
        ],
        programId: BPF_UPGRADEABLE_LOADER,
    });
}

const programId = new PublicKey(requireEnv('PROGRAM_ID'));
const programBuffer = new PublicKey(requireEnv('PROGRAM_BUFFER'));
const vault = new PublicKey(requireEnv('SQUADS_VAULT'));
const spill = new PublicKey(requireEnv('SPILL_ADDRESS'));

const verifyTx = decodeTransaction(requireEnv('VERIFY_TX'));
const verifyInstructions = withoutComputeBudget(verifyTx.instructions);
const metadataInstructions = requireEnv('METADATA_TXS')
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean)
    .flatMap(line => withoutComputeBudget(decodeTransaction(line).instructions));

if (verifyInstructions.length === 0) {
    throw new Error('Verify transaction contains no instructions');
}
if (metadataInstructions.length === 0) {
    throw new Error('Metadata transactions contain no instructions');
}

const combined = new Transaction();
combined.feePayer = vault;
combined.recentBlockhash = verifyTx.recentBlockhash ?? bs58.encode(Buffer.alloc(32));
combined.add(
    ...verifyInstructions,
    ...metadataInstructions,
    upgradeInstruction(programId, programBuffer, vault, spill),
);

const wire = combined.serialize({ requireAllSignatures: false, verifySignatures: false });
if (wire.length > MAX_TRANSACTION_BYTES) {
    throw new Error(
        `Combined transaction is ${wire.length} bytes (max ${MAX_TRANSACTION_BYTES}); ` +
            'import the exported verify and metadata transactions into Squads separately',
    );
}

console.error(
    `Combined ${verifyInstructions.length} verify + ${metadataInstructions.length} metadata + 1 upgrade instructions (${wire.length} bytes)`,
);
console.log(bs58.encode(wire));
