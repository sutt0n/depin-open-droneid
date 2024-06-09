import { useStore } from "/assets/js/Store.js";

class Drone {
  constructor(drone) {
    Object.assign(this, drone);
    this.focused = false;
    this.blurred = false;
    this.showPath = false;
    this.showPilot = false;
    this.showHome = false;
    this.history = [];
    this.flights = undefined;
  }
}

const getJsonResponse = async (url) => {
  const response = await fetch(url);
  return await response.json();
};

const getDrones = async (url) =>
  (await getJsonResponse(url)).map((it) => new Drone(it));

export const getSettings = async () => {
  return {
    google_maps_api_key: "AIzaSyDw7jvT605sN74PTADZogMb8IA3cVacbxU",
    activity_offset_in_m: 10,
    drone_size_in_rem: 5,
    interfaces: ["en0", "lo0"],
    performance_mode: true,
  };
};
//getJsonResponse("/api/settings");
export const getInterfaces = async () => {
  return [];
};
// getJsonResponse("/api/settings/interfaces");
export const getActiveDrones = async () => getDrones("/api/drones/active");
export const getAllDrones = async () => getDrones("/api/drones/all");
export const getHistory = async (serial_number) =>
  getJsonResponse(`/api/drones/${serial_number}/history`);
export const getFlights = async (serial_number) =>
  getJsonResponse(`/api/drones/${serial_number}/flights`);
export const getFlight = async (serial_number, flight_timestamp) => {
  const url_timestamp = encodeURIComponent(flight_timestamp);
  return await getJsonResponse(
    `/api/drones/${serial_number}/flights/${url_timestamp}`
  );
};

export const postSettings = async (settings) => {
  const response = await fetch("/api/settings", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(settings),
  });
  const { status } = response;
  if (status !== 200) throw new Error("Settings update failed.");
  return await response.json();
};

export const initWebSocket = () => {
  const store = useStore();
  const { updateDrone } = store;
  // continue to try to connect to websocket
  let ws = null;

  const connect = () => {
    ws = new WebSocket(`http://192.168.1.65:3000/api/stream`);
    ws.onmessage = (event) => {
      const drone = new Drone(JSON.parse(event.data).drone);
      updateDrone(drone);
    };
    ws.onclose = () => setTimeout(connect, 1000);
  };

  const max_retries = 10;

  let retries = 0;

  const reconnect = () => {
    if (retries < max_retries) {
      retries++;
      connect();
    }
  };

  try {
    connect();
  } catch (error) {
    reconnect();
  }
};
