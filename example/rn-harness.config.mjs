import {
  androidPlatform,
  androidEmulator,
} from '@react-native-harness/platform-android';
import {
  applePlatform,
  appleSimulator,
} from '@react-native-harness/platform-apple';

const config = {
  entryPoint: './index.js',
  appRegistryComponentName: 'TemporalExample',

  runners: [
    androidPlatform({
      name: 'android',
      device: androidEmulator('Pixel_9_Pro_API_35', {
        apiLevel: 35,
        profile: 'pixel_6',
        diskSize: '1G',
        heapSize: '1G',
      }),
      bundleId: 'temporal.example',
    }),
    applePlatform({
      name: 'ios',
      device: appleSimulator('iPhone 16 Pro', '26.0'),
      bundleId: 'temporal.example',
    }),
  ],
  defaultRunner: 'android',
  resetEnvironmentBetweenTestFiles: false,
};

export default config;
