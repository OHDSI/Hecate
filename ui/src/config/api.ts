import axios from "axios";

export const API_BASE_URL = "https://hecate.pantheon-hds.com/api";

// For development, uncomment the lines below:
// export const API_BASE_URL = "http://localhost:8080/api";

export const createApiClient = (baseURL: string = API_BASE_URL) => {
  return axios.create({
    baseURL,
  });
};
