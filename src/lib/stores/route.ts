import { writable } from "svelte/store";

// create a store to track the current route path
export const currentRoute = writable<string>("apto");

// set the route based on the path
export function setRoute(path: string) {
  // extract the route name from the path
  const routeName = path === "/" ? "" : path.split("/").pop() || "";

  // set the route in the format "apto" or "apto/route"
  currentRoute.set(routeName ? `apto/${routeName}` : "apto");
}
