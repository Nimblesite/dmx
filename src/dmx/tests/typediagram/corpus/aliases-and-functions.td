alias Email = String
alias UserId = Uuid
alias Callback = Option<String>
alias Index<K> = Map<K, List<Email>>

function fetch<T>(request: Request, fallback: Option<T>) -> Response

async function store(item: Request) -> Unit

function read {
  (path: String) -> Bytes
  async (path: String, timeout: Float) -> Bytes
}

async function drain {
  () -> Unit
  async (limit: Int) -> Int
}

function nothing() -> Unit

type Request { url: String }
type Response { status: Int }
