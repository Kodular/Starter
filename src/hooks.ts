import {useQuery} from "@tanstack/react-query";
import {invoke} from "@tauri-apps/api/core";

export function useServerStatus() {
  const {data, error, status} = useQuery({
    queryKey: ["is-server-running"],
    queryFn: async () => {
      try {
        const response = await fetch("http://localhost:8004/ping");
        return response.ok;
      } catch (error) {
        return false;
      }
    },
    placeholderData: false,
    refetchInterval: 3000,
    retry: false,
  });
  console.log('is-server-running', {data, error, status});
  return data;
}

type DeviceInfo = {
  serial_no: string;
  model: string;
  android_version: string;
  sdk_version: string;
}

export function useDeviceInfo() {
  const {data, error, status} = useQuery({
    queryKey: ["device-info"],
    queryFn: async () => {
      const ret = await invoke<DeviceInfo>("device_info").catch(() => null)
      console.log('invoke(device_info) returned', ret);
      return ret;
    },
    refetchInterval: 3000,
    retry: false,
  });
  console.log('device_info', {data, error, status});
  return data;
}