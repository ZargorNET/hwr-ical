import {Semester} from "./semester";

export const API_URL = import.meta.env.PROD ? "https://hwrical2.zrgr.pw" : "http://localhost:8080";

export async function get_regex_limit(): Promise<number> {
    const res = await fetch(`${API_URL}/regex_limit`);

    return (await res.json()).limit;
}

export async function get_courses(): Promise<Map<string, Semester[]>> {
    const res = await fetch(`${API_URL}/courses`);

    return (await res.json()).courses;
}
