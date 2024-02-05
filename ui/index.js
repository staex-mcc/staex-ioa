console.log("...")


import { mnemonicGenerate, mnemonicToMiniSecret, cryptoWaitReady } from "@polkadot/util-crypto";
import { Sdk } from "@peaq-network/sdk";
import { createStorageKeys } from "@peaq-network/sdk/src/utils/index.js";
import { CreateStorageKeysEnum } from "@peaq-network/sdk/src/types/index.js";
import { Keyring } from '@polkadot/keyring';

const generateMnemonicSeed = async () => {
  // const mnemonicSeed = mnemonicGenerate();
  const mnemonicSeed = ""
  await cryptoWaitReady();
  // const mnemonicSeed = "";
  console.debug(mnemonicSeed);
  const keyring = new Keyring({ type: 'sr25519' });
  const seed = mnemonicToMiniSecret(mnemonicSeed);
  console.debug(seed);
  const pair = keyring.addFromSeed(seed);
  const address = pair.address;
  console.log(address)
  return mnemonicSeed;
};

const connectToPeaqTestnet = async () => {
  const mnemonicSeed = await generateMnemonicSeed();
  console.debug("Generated seed:", mnemonicSeed)
  const sdkInstance = await Sdk.createInstance({
    baseUrl: "wss://wsspc1-qa.agung.peaq.network",
    seed: mnemonicSeed,
  });
  await sdkInstance.connect();


  const name = "peaq-sdk-test-more"
  const { hash } = await sdkInstance.did.create({
    name, customDocumentFields: {
      services: [
        {
          id: "hello---",
          type: "raw",
          data: "pizza"
        }
      ]
    }
  });
  console.log(hash)

  // const { hashed_key } = createStorageKeys([
  //   {
  //     value: "5CS3ZHVZRSKckfQ583aCszSsMiJ6F32kNUGgxTvzdTpdcrCh",
  //     type: CreateStorageKeysEnum.ADDRESS,
  //   },
  //   { value: name, type: CreateStorageKeysEnum.STANDARD },
  // ]);
  // console.log(hashed_key)
  const did = await sdkInstance.did.read({ name });
  console.log(did.document);

  await sdkInstance.disconnect();
};


const readDID = async (sdk, name) => {
  const mnemonicSeed = await generateMnemonicSeed();
  const did = await sdk.did.read(name, mnemonicSeed);
  console.log(did);
};

(async () => {
  await connectToPeaqTestnet()
})()
