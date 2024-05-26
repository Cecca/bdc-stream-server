import java.util.HashMap;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.IOException;
import java.io.DataOutputStream;
import java.io.PrintWriter;
import java.io.OutputStreamWriter;
import java.net.ServerSocket;
import java.net.Socket;
import java.util.stream.LongStream;
import java.util.stream.Stream;

class Counter {

  public static void main(String[] args) throws Exception {

    LongStream stream = initStream(34);

    HashMap<Long, Long> histogram = new HashMap();
    long LIMIT = 10000000;

    long start = System.currentTimeMillis();
    stream
      .limit(LIMIT)
      .forEach((x) -> {
        histogram.merge(x, 1L, (a, b) -> a + b);
      });
    long end = System.currentTimeMillis();
    double throughput = LIMIT / ( (end - start) / 1000.0 );
    System.out.println("Throughput: " + throughput);

    // for (HashMap.Entry<Long, Long> pair : histogram.entrySet()) {
    //   System.out.format("%8d : %d\n", pair.getKey(), pair.getValue());
    // }
  }

  static LongStream initStream(long seed) throws IOException {
    Socket socket = new Socket("algo.dei.unipd.it", 8888);

    BufferedReader reader = new BufferedReader(new InputStreamReader(socket.getInputStream()));

    PrintWriter out = new PrintWriter(new OutputStreamWriter(socket.getOutputStream()));
    String seedStr = Long.toString(seed);
    out.println(seedStr);
    out.flush();

    return reader.lines().mapToLong(Long::parseLong);

  }
}
