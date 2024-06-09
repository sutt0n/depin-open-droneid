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
  const ws = new WebSocket(`http://${window.location.host}/api/stream`);
  const sse = new EventSource(`http://${window.location.host}/api/stream`);
  sse.onmessage = (event) => {
    console.log("event", event);
    const drone = new Drone(JSON.parse(event.data).drone);
    updateDrone(drone);
  };
};
