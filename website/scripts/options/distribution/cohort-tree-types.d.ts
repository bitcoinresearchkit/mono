import type { AnySeriesPattern } from "../../modules/bitview-client/index.js";

type Split<Path extends string> = Path extends `${infer Head}.${infer Tail}`
  ? readonly [Head, ...Split<Tail>]
  : readonly [Path];

type ProjectChildren<
  Tree extends object,
  Path extends readonly PropertyKey[],
  Skip extends PropertyKey = never,
> = {
  [Key in keyof Tree as Key extends Skip
    ? never
    : ProjectCohort<Tree[Key], Path> extends never
    ? never
    : Key]: ProjectCohort<Tree[Key], Path>;
};

type DirectKey<Tree, Path extends readonly PropertyKey[]> =
  Path extends readonly [infer Head extends PropertyKey, ...PropertyKey[]]
    ? Head extends keyof Tree
      ? Head
      : Path extends readonly ["term", "short"]
        ? "short" extends keyof Tree
          ? "short"
          : "sth" extends keyof Tree
            ? "sth"
            : never
        : Path extends readonly ["term", "long"]
          ? "long" extends keyof Tree
            ? "long"
            : "lth" extends keyof Tree
              ? "lth"
              : never
          : never
    : never;

type DirectValue<Tree, Path extends readonly PropertyKey[]> =
  DirectKey<Tree, Path> extends infer Key extends keyof Tree
    ? Key extends Path[0]
      ? Path extends readonly [PropertyKey, ...infer Tail extends PropertyKey[]]
        ? ProjectCohort<Tree[Key], Tail>
        : never
      : Tree[Key]
    : never;

type MergeProjection<Direct, Children extends object> = [Direct] extends [never]
  ? keyof Children extends never
    ? never
    : Children
  : keyof Children extends never
    ? Direct
    : Direct & Children;

export type ProjectCohort<
  Tree,
  Path extends readonly PropertyKey[],
> = Path extends readonly []
  ? Tree
  : Tree extends AnySeriesPattern
    ? never
    : Path extends readonly [PropertyKey, ...PropertyKey[]]
      ? Tree extends object
        ? MergeProjection<
            DirectValue<Tree, Path>,
            ProjectChildren<Tree, Path, DirectKey<Tree, Path>>
          >
        : never
      : never;

export type ProjectCohortPath<Tree, Path extends string> = ProjectCohort<
  Tree,
  Split<Path>
>;
