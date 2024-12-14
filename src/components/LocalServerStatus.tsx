import {useQuery} from "@tanstack/react-query";

export function LocalServerStatus() {
  const {data: isServerRunning} = useQuery({
    queryKey: ["is-server-running"],
    queryFn: async () => {
      try {
        const response = await fetch("http://localhost:8004");
        return response.ok;
      } catch (error) {
        return false;
      }
    },
    initialData: false,
    refetchInterval: 1000,
  });

  return (
    <div>
      <h1>Local Server Status</h1>
      <p>{isServerRunning ? "Running" : "Not running"}</p>
    </div>
  );
}