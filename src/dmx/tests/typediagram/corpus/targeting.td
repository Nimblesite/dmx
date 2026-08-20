@targets(dart, rust)
type OnlyDartAndRust { a: Int }

@skipTargets(go)
type NotGo { b: String }

@targets(dart)
@skipTargets(python)
union Both {
  One
  Two { x: Int }
}

alias Plain = Int
