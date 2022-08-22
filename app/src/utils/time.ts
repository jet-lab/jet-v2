export const MS_PER_SECOND = 1000;
export const MS_PER_MINUTE = MS_PER_SECOND * 60;
export const MS_PER_HOUR = MS_PER_MINUTE * 60;
export const MS_PER_DAY = MS_PER_HOUR * 24;
export const MS_PER_WEEK = MS_PER_DAY * 7;
export const MS_PER_MONTH = MS_PER_DAY * 30;
export const MS_PER_YEAR = MS_PER_DAY * 365;

export const SECONDS_PER_MIN = 60;
export const SECONDS_PER_HOUR = SECONDS_PER_MIN * 60;
export const SECONDS_PER_DAY = SECONDS_PER_HOUR * 24;
export const SECONDS_PER_WEEK = SECONDS_PER_DAY * 7;
export const SECONDS_PER_MONTH = SECONDS_PER_DAY * 30;
export const SECONDS_PER_YEAR = SECONDS_PER_DAY * 365;

export function unixToLocalTime(unixTimestamp: number) {
  const date = new Date(unixTimestamp);
  let hours = date.getHours();
  const min = ('0' + date.getMinutes()).slice(-2);
  let amPm = 'AM';

  if (hours > 12) {
    hours = hours - 12;
    amPm = 'PM';
  } else if (hours === 12) {
    hours = 12;
    amPm = 'PM';
  } else if (hours === 0) {
    hours = 12;
  }

  return `${hours}:${min} ${amPm}`;
}

export function unixToUtcTime(unixTimestamp: number) {
  const date = new Date(unixTimestamp);
  const hours = date.getHours();
  const minutes = '0' + date.getMinutes();
  const seconds = '0' + date.getSeconds();
  return `${hours}:${minutes.substr(-2)}:${seconds.substr(-2)} UTC`;
}

export function localDayMonthYear(timestamp: number, preferDayMonthYear: boolean) {
  const date = new Date(timestamp);
  const day = date.getDate();
  const month = date.getMonth() + 1;
  const year = date.getFullYear();
  if (preferDayMonthYear) {
    return `${day}/${month}/${year}`;
  } else {
    return `${month}/${day}/${year}`;
  }
}
export function utcDayMonthYear(timestamp: number, preferDayMonthYear: boolean) {
  const date = new Date(timestamp);
  const day = date.getUTCDate();
  const month = date.getUTCMonth() + 1;
  const year = date.getUTCFullYear();
  if (preferDayMonthYear) {
    return `${day}/${month}/${year}`;
  } else {
    return `${month}/${day}/${year}`;
  }
}
