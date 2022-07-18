import { promises as fs } from "fs"
import path from "path"
async function main() {
  const idlDir = path.resolve(path.join(process.cwd(), ".."))
  const targetIdlDir = path.resolve(path.join(idlDir, "../../../target/idl"))
  const targetTypesDir = path.resolve(path.join(idlDir, "../../../taget/types"))
  const outDir = path.resolve(path.join(idlDir, "../ts/types"))

  console.log("Merging IDLs")
  console.log(`IDL dir ${idlDir}`)
  console.log(`Target IDL dir ${targetIdlDir}`)
  console.log(`Target Types dir ${targetTypesDir}`)
  console.log(`Out dir ${outDir}`)

  const idlFileNames = await fs.readdir(idlDir)

  for (const idlFileName of idlFileNames) {
    const idlPath = path.join(idlDir, idlFileName)
    const targetIdlPath = path.join(targetIdlDir, idlFileName)
    const targetTypesPath = path.join(targetTypesDir, idlFileName)
    const outPath = path.join(outDir, idlFileName)

    const idlFile = JSON.parse(await fs.readFile(idlPath, "utf8"))
    const targetIdlFile = JSON.parse(await fs.readFile(targetIdlPath, "utf8"))

    const mergedIdlFile = mergeDeep(targetIdlFile, idlFile)
  }
}
main()

/**
 * Simple object check.
 * @param item
 * @returns {boolean}
 */
function isObject(item: any) {
  return item && typeof item === "object" && !Array.isArray(item)
}

/**
 * Deep merge two objects.
 * @param target
 * @param ...sources
 */
function mergeDeep(target: any, source: any) {
  if (isObject(target) && isObject(source)) {
    for (const key in source) {
      if (isObject(source[key])) {
        if (!target[key]) Object.assign(target, { [key]: {} })
        mergeDeep(target[key], source[key])
      } else {
        Object.assign(target, { [key]: source[key] })
      }
    }
  }
}
