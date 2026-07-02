import { app } from 'electron';

const DISABLED_GPU_SWITCHES = [
  'disable-gpu',
  'disable-gpu-compositing',
  'disable-gpu-rasterization',
  'disable-gpu-sandbox',
  'disable-accelerated-2d-canvas',
  'disable-accelerated-video-decode',
] as const;

export function configureGpu(): void {
  app.commandLine.appendSwitch('disable-logging');
  app.commandLine.appendSwitch('log-level', '3');

  app.disableHardwareAcceleration();

  for (const switchName of DISABLED_GPU_SWITCHES) {
    app.commandLine.appendSwitch(switchName);
  }
}
