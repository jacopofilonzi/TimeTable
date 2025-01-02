export default function ParseDate(date: Date): string {
    return `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${date.getUTCFullYear()} - ${date.getUTCHours()}:${date.getUTCMinutes()}:${date.getUTCSeconds()}`
}