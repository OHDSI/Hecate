import axios from "axios";

export const API_BASE_URL = "https://hecate.pantheon-hds.com/api";
export const API_V2_BASE_URL = "https://hecate.pantheon-hds.com/v2";

// For development, uncomment the lines below:
// export const API_BASE_URL = "http://localhost:8080/api";
// export const API_V2_BASE_URL = "http://localhost:8081/v2";

export const createApiClient = (baseURL: string = API_BASE_URL) => {
  return axios.create({
    baseURL,
  });
};
