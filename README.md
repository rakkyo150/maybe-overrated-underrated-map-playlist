![Build](https://github.com/rakkyo150/maybe-overrated-underrated-map-playlist/actions/workflows/main.yml/badge.svg)

# maybe-overrated-underrated-map-playlist
[PredictStarNumber](https://github.com/rakkyo150/PredictStarNumber)による星予測と実際の星予測の差ごとのプレイリストを星ごとに作成するリポジトリ

## 注意
あくまで星予測値との差であり、正確に譜面の過小評価や過大評価ができていることが保障されているわけではありません。  
そのため、実用性はあまりないかもしれません。  

## 使い方
前提として、Beat Saberでカスタムプレイリストが使える環境が必要です。  
[Releases](https://github.com/rakkyo150/maybe-overrated-underrated-map-playlist/releases)のall.zipをダウンロードし展開してください。  
欲しいプレイリストが決まっている場合は、jsonファイルを直接ダウンロードしてもOKです。  
Beat Saber/Playlistsにjsonファイルを入れてください。  
Beat Saberを開いてPlaylistsフォルダに入れたプレイリストを見つけられれば成功です。

## プレイリストの内容
プレイリストは、上から予測値との差が大きい順にソートされています。  
overratedは実際の値の方が大きいのを意味し、underratedは実際の値の方が小さいのを意味します。  
a_littleは予測値との差が0.5未満の場合、fairlyは0.5以上1未満、veryは1以上を意味します。